use bumpalo_herd::Herd;

use bumpalo_herd::Member;

use std::mem::MaybeUninit;

use super::Bound;

use crate::generic_tree::Distance;
use crate::tree_data::TreeData;

use num::Float;

/// Node
/// ------------
///
/// How does the coord work?
///
/// <img src="https://i.ibb.co/mcTpnF2/image.png" width="300"/>
///
/// let m represent the middle of the bound. how do we infer a region's index in parent.children?
///
/// ```txt
/// (x, y, z)
/// -> (t0 = x>m0, t1 = y>m1, t2 = z>m2)
/// -> t0 + 2*t1 + 4*t2
/// ```
#[derive(Debug)]
pub enum Node<'bump, F: Float + Send + Sync, const N: usize, const N2: usize, D: TreeData> {
    Point {
        coord: [F; N],
        data: D::PointData,
    },
    Region {
        bounds: [Bound<F>; N],
        children: [Option<&'bump mut Node<'bump, F, N, N2, D>>; N2],
        data: D::RegionData,
    },
}

#[inline]
pub(crate) fn two_power(n: usize) -> usize {
    1 << n
}

impl<'bump, F: Float + Send + Sync, const N: usize, const N2: usize, D: TreeData>
    Node<'bump, F, N, N2, D>
{
    pub fn new_region(bounds: [Bound<F>; N]) -> Self {
        Node::Region {
            bounds: bounds,
            children: unsafe {
                let mut arr: [Option<_>; N2] = MaybeUninit::uninit().assume_init();
                for item in &mut arr[..] {
                    std::ptr::write(item, None);
                }
                arr
            },
            data: D::RegionData::default(),
        }
    }

    pub fn try_get_children(
        &mut self,
    ) -> Option<&mut [Option<&'bump mut Node<'bump, F, N, N2, D>>; N2]> {
        match self {
            Node::Point { coord: _, data: _ } => None,
            Node::Region { children, .. } => Some(children),
        }
    }

    pub fn has_children(&self) -> bool {
        match self {
            Node::Point { coord: _, data: _ } => false,
            Node::Region { children, .. } => children.iter().any(|child| child.is_some()),
        }
    }

    pub fn new_point(coord: [F; N], data: D::PointData) -> Self {
        Node::Point { coord, data }
    }

    pub fn is_region(&self) -> bool {
        match self {
            Node::Point { coord: _, data: _ } => false,
            Node::Region { .. } => true,
        }
    }

    pub fn distance(&self, point: &[F; N]) -> F {
        if self.contains(point) {
            return F::zero();
        }

        match self {
            Node::Point { coord, data: _ } => coord.dist(point),
            Node::Region { bounds, .. } => {
                let mut dist = F::zero();
                for i in 0..N {
                    if point[i] > bounds[i].max {
                        dist = dist + (point[i] - bounds[i].max).powi(2);
                    } else if point[i] < bounds[i].min {
                        dist = dist + (bounds[i].min - point[i]).powi(2);
                    }
                }

                F::sqrt(dist)
            }
        }
    }

    pub fn is_leaf_region(&self) -> bool {
        match self {
            Node::Point { coord: _, data: _ } => false,
            Node::Region { children, .. } => {
                if let Some(child) = &children[0] {
                    !child.is_region()
                } else {
                    true
                }
            }
        }
    }

    pub fn divide(&mut self, member: &Member<'bump>) -> Result<(), ()> {
        match self {
            Node::Region {
                bounds, children, ..
            } => {
                if children.iter().filter(|x| x.is_some()).count() == 0 {
                    let mut children_bounds = vec![];
                    for _ in 0..two_power(N) {
                        children_bounds.push(bounds.clone());
                    }

                    for i in 0..two_power(N) {
                        for j in 0..N {
                            if (i & (1 << j)) > 0 {
                                children_bounds[i][j].min = bounds[j].middle();
                            } else {
                                children_bounds[i][j].max = bounds[j].middle();
                            }
                        }
                    }

                    for (i, child) in children_bounds.iter().enumerate() {
                        children[i] = Some(member.alloc(Node::new_region(child.clone())));
                    }
                    Ok(())
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }

    pub fn contains(&self, point: &[F; N]) -> bool {
        match self {
            Node::Point { coord: _, data: _ } => false,
            Node::Region { bounds, .. } => {
                for i in 0..N {
                    if point[i] < bounds[i].min || point[i] > bounds[i].max {
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }

    pub fn get_child_region_index(point: &[F; N], bounds: &[Bound<F>; N]) -> usize {
        let mut index = 0;
        for i in 0..N {
            if point[i] > bounds[i].middle() {
                index = index + (1 << i);
            }
        }

        index
    }

    pub fn get_sub_region(&self, point: &[F; N]) -> usize {
        match self {
            Node::Region { bounds, .. } => {
                let mut index = 0;
                for i in (0..N).rev() {
                    let m = bounds[i].middle();
                    if point[i] > m {
                        index += 1;
                    }
                    index <<= 1;
                }

                index >> 1
            }
            _ => panic!(),
        }
    }

    pub(crate) fn get_leaf_region(&mut self, point: &[F; N]) -> &mut Self {
        let mut node = self;
        while !node.is_leaf_region() {
            let index = node.get_sub_region(&point);
            match node {
                Node::Region { children, .. } => {
                    node = children[index].as_mut().unwrap();
                }
                _ => panic!(),
            }
        }

        node
    }

    pub(crate) fn insert_point_directly(&mut self, point: &'bump mut Self) {
        match self {
            Node::Point { coord: _, data: _ } => {}
            Node::Region { children, .. } => {
                for i in 0..children.len() {
                    if children[i].is_none() {
                        children[i] = Some(point);
                        break;
                    }
                }
            }
        }
    }

    pub(crate) fn child_len(&self) -> usize {
        match self {
            Node::Point { coord: _, data: _ } => 0,
            Node::Region { children, .. } => children.iter().filter(|x| x.is_some()).count(),
        }
    }

    pub(crate) fn insert_point(
        &mut self,
        herd: &'bump Herd,
        point: &'bump mut Self,
        max_num: u32,
    ) -> Result<(), ()> {
        if !self.is_leaf_region() {
            return Err(());
        }

        if !self.contains(point.coord()) {
            panic!();
        }

        match self {
            Node::Point { coord: _, data: _ } => return Err(()),
            Node::Region { children, .. } => {
                for i in 0..children.len() {
                    if children[i].is_none() {
                        children[i] = Some(point);
                        break;
                    }
                }
            }
        }

        let child_len = self.child_len();
        let should_divide = child_len as u32 > max_num || child_len >= N2;
        if should_divide {
            let mut points = vec![];
            if let Node::Region { children, .. } = self {
                for node in children.iter_mut() {
                    if node.is_some() {
                        points.push(node.take().unwrap());
                    }
                }
            }

            self.divide(&herd.get()).unwrap_or(());
            for point in points {
                match *point {
                    Node::Point { coord, data: _ } => {
                        let node = Some(self.get_leaf_region(&coord));
                        node.unwrap().insert_point(herd, point, max_num)?;
                    }
                    Node::Region { .. } => panic!(),
                }
            }
        }

        Ok(())
    }

    pub fn bounds(&self) -> &[Bound<F>; N] {
        match self {
            Node::Point { coord: _, data: _ } => panic!(),
            Node::Region { bounds, .. } => bounds,
        }
    }

    pub(crate) fn coord(&self) -> &[F; N] {
        match self {
            Node::Point { coord, data: _ } => coord,
            _ => panic!(),
        }
    }

    pub fn data(&self) -> &D::PointData {
        match self {
            Node::Point { coord: _, data } => data,
            _ => panic!(),
        }
    }

    pub fn region_data(&self) -> &D::RegionData {
        match self {
            Node::Region { data, .. } => data,
            _ => panic!(),
        }
    }

    pub fn set_data(&mut self, value: D::PointData) {
        match self {
            Node::Point { coord: _, data } => {
                *data = value;
            }
            _ => panic!(),
        }
    }

    pub fn visit_post_order<FF>(&self, func: &mut FF)
    where
        FF: FnMut(&Self) -> (),
    {
        if self.is_region() {
            if let Node::Region { children, .. } = self {
                for child in children.iter() {
                    if let Some(child) = child {
                        child.visit_post_order(func);
                    }
                }
            }
        }

        func(self);
    }

    #[cfg(not(debug_assertions))]
    pub(crate) fn check(&self) -> Result<(), ()> {
        Ok(())
    }

    #[cfg(debug_assertions)]
    pub(crate) fn check(&self) -> Result<(), ()> {
        match self {
            Node::Point { coord: _, data: _ } => Ok(()),
            Node::Region {
                bounds, children, ..
            } => {
                if children.len() == 0 {
                    for i in 0..N {
                        assert!(bounds[i].min < bounds[i].max);
                    }
                } else {
                    if self.is_leaf_region() {
                        for child in children.iter().filter(|c| c.is_some()) {
                            assert!(!child.as_ref().unwrap().is_region());
                            assert!(self.contains(child.as_ref().unwrap().coord()));
                        }
                    } else {
                        for child in children {
                            assert!(child.as_ref().unwrap().is_region());
                        }

                        assert_eq!(children.len(), two_power(N));
                    }

                    for child in children.iter().filter(|c| c.is_some()) {
                        child.as_ref().unwrap().check()?;
                    }
                }

                Ok(())
            }
        }
    }

    pub(crate) fn children(&mut self) -> &mut [Option<&'bump mut Node<'bump, F, N, N2, D>>; N2] {
        match self {
            Node::Point { coord: _, data: _ } => panic!(),
            Node::Region { children, .. } => children,
        }
    }
}
