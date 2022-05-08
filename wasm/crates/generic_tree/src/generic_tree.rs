use core::panic;
use rayon::{join, prelude::*, ThreadPoolBuilder};
use std::{fmt::Display, time::Instant};

use num::Float;

use crate::tree_data::TreeData;

/// Bounds
///
///
///
#[derive(Clone, Copy, Debug)]
pub struct Bound<F: Float> {
    pub min: F,
    pub max: F,
}

impl<F: Float + Display> Display for Bound<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[min: {}, max: {}]", self.min, self.max)
    }
}

impl<F: Float> Bound<F> {
    pub fn width(&self) -> F {
        self.max - self.min
    }

    pub fn middle(&self) -> F {
        (self.max + self.min) / F::from(2).unwrap()
    }

    pub fn contains(&self, v: F) -> bool {
        v >= self.min && v < self.max
    }
}

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
pub enum Node<F: Float, const N: usize, D: TreeData> {
    Point {
        coord: [F; N],
        data: D::PointData,
    },
    Region {
        bounds: [Bound<F>; N],
        children: Vec<Box<Node<F, N, D>>>,
        data: D::RegionData,
    },
}

impl<F: Float, const N: usize, D: TreeData> Clone for Node<F, N, D> {
    fn clone(&self) -> Self {
        match self {
            Self::Point { coord, data } => Self::Point {
                coord: coord.clone(),
                data: data.clone(),
            },
            Self::Region {
                bounds,
                children,
                data,
            } => Self::Region {
                data: data.clone(),
                bounds: bounds.clone(),
                children: children.clone(),
            },
        }
    }
}

#[inline]
fn two_power(n: usize) -> usize {
    1 << n
}

impl<F: Float, const N: usize, D: TreeData> Node<F, N, D> {
    pub fn new_region(bounds: [Bound<F>; N]) -> Self {
        Node::Region {
            bounds,
            children: vec![],
            data: D::RegionData::default(),
        }
    }

    pub fn try_get_children(&mut self) -> Option<&mut Vec<Box<Node<F, N, D>>>> {
        match self {
            Node::Point { coord: _, data: _ } => None,
            Node::Region { children, .. } => Some(children),
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
                if children.len() == 0 {
                    true
                } else {
                    !children[0].is_region()
                }
            }
        }
    }

    pub fn divide(&mut self) -> Result<(), ()> {
        match self {
            Node::Region {
                bounds, children, ..
            } => {
                if children.len() == 0 {
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

                    for child in children_bounds.iter() {
                        children.push(Box::new(Node::new_region(child.clone())));
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

    fn get_leaf_region(&mut self, point: &[F; N]) -> &mut Self {
        let mut node = self;
        while !node.is_leaf_region() {
            let index = node.get_sub_region(&point);
            match node {
                Node::Region { children, .. } => {
                    node = &mut children[index];
                }
                _ => panic!(),
            }
        }

        node
    }

    fn insert_points(&mut self, points: Vec<Box<Self>>, max_num: u32) -> Result<(), ()> {
        for point in points.into_iter() {
            if !self.contains(point.coord()) {
                return Err(());
            }

            let region = self.get_leaf_region(point.coord());
            region.insert_point(point, max_num).unwrap();
        }

        Ok(())
    }

    fn insert_point_directly(&mut self, point: Box<Self>) {
        match self {
            Node::Point { coord: _, data: _ } => {}
            Node::Region { children, .. } => {
                children.push(point);
            }
        }
    }

    fn insert_point(&mut self, point: Box<Self>, max_num: u32) -> Result<(), ()> {
        if !self.is_leaf_region() {
            return Err(());
        }

        let mut should_divide = false;
        if !self.contains(point.coord()) {
            panic!();
        }

        match self {
            Node::Point { coord: _, data: _ } => return Err(()),
            Node::Region { children, .. } => {
                children.push(point);
                if children.len() > max_num as usize {
                    should_divide = true;
                }
            }
        }

        if should_divide {
            let mut points = vec![];
            if let Node::Region { children, .. } = self {
                while let Some(node) = children.pop() {
                    points.push(node);
                }
            }

            self.divide().unwrap_or(());
            for point in points {
                match &*point {
                    Node::Point { coord, data: _ } => {
                        let node = Some(self.get_leaf_region(&coord));
                        node.unwrap().insert_point(point, max_num)?;
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

    fn coord(&self) -> &[F; N] {
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

    pub fn set_data(&mut self, value: D::PointData) {
        match self {
            Node::Point { coord: _, data } => {
                *data = value;
            }
            _ => panic!(),
        }
    }

    #[cfg(not(debug_assertions))]
    fn check(&self) -> Result<(), ()> {
        Ok(())
    }

    #[cfg(debug_assertions)]
    fn check(&self) -> Result<(), ()> {
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
                        for child in children {
                            assert!(!child.is_region());
                            assert!(self.contains(child.coord()));
                        }
                    } else {
                        for child in children {
                            assert!(child.is_region());
                        }

                        assert_eq!(children.len(), two_power(N));
                    }

                    for child in children {
                        child.check()?;
                    }
                }

                Ok(())
            }
        }
    }

    fn children(&mut self) -> &mut Vec<Box<Self>> {
        match self {
            Node::Point { coord: _, data: _ } => panic!(),
            Node::Region { children, .. } => children,
        }
    }
}

pub struct GenericTree<F: Float + Send + Sync, const N: usize, D: TreeData> {
    root: Node<F, N, D>,
    bounds: [Bound<F>; N],
    pub num: u32,
    min_dist: F,
    /**
     * leaf region max children
     */
    leaf_max_children: u32,
}

impl<F: Float + Send + Sync, const N: usize, D: TreeData> GenericTree<F, N, D> {
    pub fn new(bounds: [Bound<F>; N], min_dist: F, leaf_max_children: u32) -> Self {
        if leaf_max_children == 0 {
            panic!("leaf_max_children must be greater than 0");
        }

        GenericTree {
            root: Node::new_region(bounds.clone()),
            bounds,
            num: 0,
            min_dist,
            leaf_max_children,
        }
    }

    pub fn add_node(&mut self, node: Node<F, N, D>) -> Result<(), ()> {
        self.num += 1;
        if !self.root.contains(node.coord()) {
            return Err(());
        }

        let region = self.root.get_leaf_region(node.coord());
        region
            .insert_point(Box::new(node), self.leaf_max_children)
            .unwrap();

        Ok(())
    }

    pub fn add(&mut self, point: [F; N], data: D::PointData) -> Result<(), ()> {
        self.num += 1;
        if !self.root.contains(&point) {
            return Err(());
        }

        let node = self.root.get_leaf_region(&point);
        node.insert_point(
            Box::new(Node::new_point(point, data)),
            self.leaf_max_children,
        )
        .unwrap();

        Ok(())
    }
    pub fn find_closest_with_max_dist(
        &self,
        point: &[F; N],
        max_dist: F,
    ) -> Option<&Node<F, N, D>> {
        let mut stack = vec![&self.root];
        let mut min_dist = max_dist;
        let mut min_ans = None;
        while let Some(node) = stack.pop() {
            match node {
                Node::Point { coord, data: _ } => {
                    let dist = coord.dist(point);
                    if dist < min_dist {
                        min_dist = dist;
                        min_ans = Some(node);
                    }
                }
                Node::Region { children, .. } => {
                    for child in children.iter() {
                        let dist = child.distance(point);
                        if dist < min_dist {
                            stack.push(child);
                        }
                    }

                    ()
                }
            }
        }

        min_ans
    }

    pub fn find_closest(&self, point: &[F; N]) -> Option<&Node<F, N, D>> {
        self.find_closest_with_max_dist(point, F::infinity())
    }

    pub fn visit<FF>(&self, func: FF) -> ()
    where
        FF: Fn(&Node<F, N, D>, usize) -> bool,
    {
        let mut stack = vec![(&self.root, 0)];
        while let Some((node, depth)) = stack.pop() {
            if func(&node, depth) {
                return;
            }

            match node {
                Node::Point { coord: _, data: _ } => {}
                Node::Region { children, .. } => {
                    for child in children {
                        stack.push((child, depth + 1));
                    }
                }
            }
        }
    }
}

impl<F: Float + Display + Send + Sync, const N: usize, D: TreeData> GenericTree<F, N, D> {
    fn debug(&self) {
        let space = String::from(" ");
        self.visit(|node, depth| match node {
            Node::Point { coord, data } => {
                let mut s = String::new();
                for i in 0..N {
                    s += &format!("{} ", coord[i]);
                }

                print!("{}", space.repeat(depth * 4));
                println!("Point {{coord: {}, data: {}}}", s, data);
                false
            }
            Node::Region {
                bounds, children, ..
            } => {
                let mut s = String::new();
                for i in 0..N {
                    s += &format!("{} ", bounds[i]);
                }

                print!("{}", space.repeat(depth * 4));
                println!("Region {{bounds: {}, children: {}}}", s, children.len());
                false
            }
        })
    }
}
trait Distance<F: Float> {
    fn dist(&self, another: &Self) -> F;
}

impl<F: Float + Sync + Send, const N: usize, D: TreeData> GenericTree<F, N, D> {
    pub fn from_nodes(nodes: Vec<Node<F, N, D>>, min_dist: F, leaf_max_children: u32) -> Self {
        let (max, min) = nodes.iter().fold(
            ([F::neg_infinity(); N], [F::infinity(); N]),
            |(mut max_coord, mut min_coord), node| {
                let coord = node.coord();
                for i in 0..N {
                    max_coord[i] = max_coord[i].max(coord[i]);
                    min_coord[i] = min_coord[i].min(coord[i]);
                }

                (max_coord, min_coord)
            },
        );

        let bounds: [Bound<F>; N] = min
            .into_iter()
            .zip(max)
            .map(|(min, max)| Bound { min, max })
            .collect::<Vec<Bound<F>>>()
            .try_into()
            .unwrap_or_else(|_| panic!());

        let mut tree: GenericTree<F, N, D> = GenericTree::new(bounds, min_dist, leaf_max_children);
        tree.num = nodes.len() as u32;

        let mut nodes = nodes
            .into_iter()
            .map(|node| Box::new(node))
            .collect::<Vec<_>>();

        run(&mut nodes, &mut tree.root, leaf_max_children);
        std::mem::forget(nodes);
        return tree;

        fn run<F: Float + Send + Sync, const N: usize, D: TreeData>(
            nodes: &mut [Box<Node<F, N, D>>],
            leaf: &mut Node<F, N, D>,
            leaf_max_children: u32,
        ) {
            if leaf.children().len() + nodes.len() <= leaf_max_children as usize {
                for node in nodes {
                    let node: *const Box<Node<F, N, D>> = &*node;
                    unsafe {
                        leaf.insert_point_directly(std::ptr::read(node));
                    }
                }
            } else {
                let sub_nodes = divide(nodes, leaf.bounds(), N - 1);
                leaf.divide().unwrap_or(());
                leaf.children()
                    .into_iter()
                    .zip(sub_nodes)
                    .for_each(|(child, nodes)| run(nodes, child, leaf_max_children));
            }
        }

        fn divide<'a, F: Float + Send + Sync, const N: usize, D: TreeData>(
            nodes: &'a mut [Box<Node<F, N, D>>],
            bounds: &[Bound<F>; N],
            bound_index: usize,
        ) -> Vec<&'a mut [Box<Node<F, N, D>>]> {
            let mut ans = vec![];
            let middle = bounds[bound_index].middle();
            let mut lt_end_index = 0;
            for i in 0..nodes.len() {
                if nodes[i].coord()[bound_index] < middle {
                    nodes.swap(i, lt_end_index);
                    lt_end_index += 1;
                }
            }

            let (left, right) = nodes.split_at_mut(lt_end_index);
            if bound_index != 0 {
                let (left, right) = (
                    divide(left, bounds, bound_index - 1),
                    divide(right, bounds, bound_index - 1),
                );
                ans = left;
                ans.extend(right);
            } else {
                ans.push(left);
                ans.push(right);
            }

            ans
        }
    }

    pub fn new_single_thread(
        mut nodes: Vec<Box<Node<F, N, D>>>,
        min_dist: F,
        leaf_max_children: u32,
    ) -> GenericTree<F, N, D> {
        let (max, min) = nodes.iter().fold(
            ([F::neg_infinity(); N], [F::infinity(); N]),
            |(mut max_coord, mut min_coord), node| {
                let coord = node.coord();
                for i in 0..N {
                    max_coord[i] = max_coord[i].max(coord[i]);
                    min_coord[i] = min_coord[i].min(coord[i]);
                }

                (max_coord, min_coord)
            },
        );

        let bounds: [Bound<F>; N] = min
            .into_iter()
            .zip(max)
            .map(|(min, max)| Bound { min, max })
            .collect::<Vec<Bound<F>>>()
            .try_into()
            .unwrap_or_else(|_| panic!());

        let mut tree: GenericTree<F, N, D> = GenericTree::new(bounds, min_dist, leaf_max_children);
        tree.num = nodes.len() as u32;

        run(&mut nodes, &mut tree.root, leaf_max_children, 0);
        std::mem::forget(nodes);
        return tree;

        fn run<F: Float + Send + Sync, const N: usize, D: TreeData>(
            nodes: &mut [Box<Node<F, N, D>>],
            leaf: &mut Node<F, N, D>,
            leaf_max_children: u32,
            depth: usize,
        ) {
            debug_assert!(leaf.is_leaf_region());

            if leaf.children().len() + nodes.len() <= leaf_max_children as usize {
                for node in nodes {
                    let node: *const Box<Node<F, N, D>> = &*node;
                    unsafe {
                        leaf.insert_point_directly(std::ptr::read(node));
                    }
                }
            } else {
                let sub_nodes = divide(nodes, leaf.bounds(), N - 1);
                leaf.divide().unwrap_or(());
                leaf.children()
                    .into_iter()
                    .zip(sub_nodes)
                    .for_each(|(child, nodes)| run(nodes, child, leaf_max_children, depth + 1));
            }
        }

        fn divide<'a, F: Float + Send + Sync, const N: usize, D: TreeData>(
            nodes: &'a mut [Box<Node<F, N, D>>],
            bounds: &[Bound<F>; N],
            bound_index: usize,
        ) -> Vec<&'a mut [Box<Node<F, N, D>>]> {
            let mut ans = vec![];
            let middle = bounds[bound_index].middle();
            let mut lt_end_index = 0;
            for i in 0..nodes.len() {
                if nodes[i].coord()[bound_index] < middle {
                    nodes.swap(i, lt_end_index);
                    lt_end_index += 1;
                }
            }

            let (left, right) = nodes.split_at_mut(lt_end_index);
            if bound_index != 0 {
                let (left, right) = (
                    divide(left, bounds, bound_index - 1),
                    divide(right, bounds, bound_index - 1),
                );
                ans = left;
                ans.extend(right);
            } else {
                ans.push(left);
                ans.push(right);
            }

            ans
        }
    }

    pub fn new_in_par(
        mut nodes: Vec<Box<Node<F, N, D>>>,
        min_dist: F,
        leaf_max_children: u32,
    ) -> GenericTree<F, N, D> {
        let (max, min) = nodes
            .par_iter()
            .with_min_len(1000)
            .fold(
                || ([F::neg_infinity(); N], [F::infinity(); N]),
                |(mut max_coord, mut min_coord), node| {
                    let coord = node.coord();
                    for i in 0..N {
                        max_coord[i] = max_coord[i].max(coord[i]);
                        min_coord[i] = min_coord[i].min(coord[i]);
                    }

                    (max_coord, min_coord)
                },
            )
            .reduce(
                || ([F::neg_infinity(); N], [F::infinity(); N]),
                |(max_coord1, min_coord1), (max_coord2, min_coord2)| {
                    let mut min_coord = [F::neg_infinity(); N];
                    let mut max_coord = [F::infinity(); N];
                    for i in 0..N {
                        min_coord[i] = max_coord1[i].max(max_coord2[i]);
                        max_coord[i] = min_coord1[i].min(min_coord2[i]);
                    }

                    (min_coord, max_coord)
                },
            );

        let bounds: [Bound<F>; N] = min
            .into_iter()
            .zip(max)
            .map(|(min, max)| Bound { min, max })
            .collect::<Vec<Bound<F>>>()
            .try_into()
            .unwrap_or_else(|_| panic!());

        let mut tree: GenericTree<F, N, D> = GenericTree::new(bounds, min_dist, leaf_max_children);
        tree.num = nodes.len() as u32;

        run(&mut nodes, &mut tree.root, leaf_max_children, 0);
        std::mem::forget(nodes);
        return tree;

        fn run<F: Float + Send + Sync, const N: usize, D: TreeData>(
            nodes: &mut [Box<Node<F, N, D>>],
            leaf: &mut Node<F, N, D>,
            leaf_max_children: u32,
            depth: usize,
        ) {
            debug_assert!(leaf.is_leaf_region());

            if leaf.children().len() + nodes.len() <= leaf_max_children as usize {
                for node in nodes {
                    let node: *const Box<Node<F, N, D>> = &*node;
                    unsafe {
                        leaf.insert_point_directly(std::ptr::read(node));
                    }
                }
            } else {
                let sub_nodes = divide(nodes, leaf.bounds(), N - 1);
                leaf.divide().unwrap_or(());
                if depth % 3 == 0 {
                    leaf.children()
                        .into_par_iter()
                        .zip(sub_nodes)
                        .for_each(|(child, nodes)| run(nodes, child, leaf_max_children, depth + 1));
                } else {
                    leaf.children()
                        .into_iter()
                        .zip(sub_nodes)
                        .for_each(|(child, nodes)| run(nodes, child, leaf_max_children, depth + 1));
                }
            }
        }

        fn divide<'a, F: Float + Send + Sync, const N: usize, D: TreeData>(
            nodes: &'a mut [Box<Node<F, N, D>>],
            bounds: &[Bound<F>; N],
            bound_index: usize,
        ) -> Vec<&'a mut [Box<Node<F, N, D>>]> {
            let mut ans = vec![];
            let middle = bounds[bound_index].middle();
            let mut lt_end_index = 0;
            for i in 0..nodes.len() {
                if nodes[i].coord()[bound_index] < middle {
                    nodes.swap(i, lt_end_index);
                    lt_end_index += 1;
                }
            }

            let (left, right) = nodes.split_at_mut(lt_end_index);
            if bound_index != 0 {
                let (left, right) = join(
                    || divide(left, bounds, bound_index - 1),
                    || divide(right, bounds, bound_index - 1),
                );
                ans = left;
                ans.extend(right);
            } else {
                ans.push(left);
                ans.push(right);
            }

            ans
        }
    }

    fn clear(&mut self) {
        if let Some(stack) = self.root.try_get_children() {
            while let Some(node) = stack.pop() {
                match *node {
                    Node::Point { .. } => {}
                    Node::Region { mut children, .. } => stack.append(&mut children),
                }
            }
        }
        self.num = 0;
    }
}

impl<F: Float, const N: usize> Distance<F> for [F; N] {
    fn dist(&self, another: &Self) -> F {
        let mut square_sum = F::zero();
        for i in 0..N {
            square_sum = square_sum + (self[i] - another[i]).powi(2);
        }

        F::sqrt(square_sum)
    }
}

mod tests {
    use std::thread;

    use crate::tree_data::TreeData;

    use super::{Bound, GenericTree, Node};
    struct Data;
    impl TreeData for Data {
        type PointData = usize;
        type RegionData = usize;
    }

    #[test]
    fn test_debug() {
        let mut tree: GenericTree<f64, 2, Data> = GenericTree::new(
            [
                Bound {
                    min: -1.0,
                    max: 101.0,
                },
                Bound {
                    min: -1.0,
                    max: 101.0,
                },
            ],
            0.1,
            1,
        );

        for i in 0..10 {
            for j in 0..10 {
                tree.add([(i * 10) as f64, (j * 10) as f64], i * 100 + j)
                    .unwrap();
                tree.root.check().unwrap();
            }
        }

        tree.debug();
    }

    #[test]
    fn test_add() {
        let mut tree: GenericTree<f64, 2, Data> = GenericTree::new(
            [
                Bound {
                    min: -10.0,
                    max: 101.0,
                },
                Bound {
                    min: -10.0,
                    max: 101.0,
                },
            ],
            0.1,
            10,
        );

        for i in 0..100 {
            for j in 0..100 {
                tree.add([i as f64, j as f64], i * 100 + j).unwrap();
                tree.root.check().unwrap();
            }
        }

        for i in 0..100 {
            for j in 0..100 {
                let temp = tree.find_closest(&[i as f64, j as f64]).unwrap();
                assert_eq!(temp.data(), &(i * 100 + j));
            }
        }
    }

    #[test]
    fn test_parallel_inserts() {
        let mut nodes = vec![];
        for i in 0..100 {
            for j in 0..100 {
                nodes.push(Box::new(Node::new_point(
                    [i as f64, j as f64],
                    i * 100 + j as usize,
                )));
            }
        }

        let tree = GenericTree::<f64, 2, Data>::new_in_par(nodes, 1.0, 10);
        tree.root.check().unwrap();
        for i in 0..100 {
            for j in 0..100 {
                assert_eq!(
                    *tree
                        .find_closest_with_max_dist(&[i as f64, j as f64], 2.0)
                        .unwrap()
                        .data(),
                    i * 100 + j
                );
            }
        }
    }
}
