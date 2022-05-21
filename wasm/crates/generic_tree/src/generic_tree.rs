use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use bumpalo_herd::{Herd, Member};
use core::panic;
use rayon::{join, prelude::*, ThreadPoolBuilder};
use std::{fmt::Display, mem::ManuallyDrop, time::Instant};

use num::Float;

use crate::{tree_data::TreeData, Node};

/// Bounds
#[derive(Clone, Copy, Debug)]
pub struct Bound<F: Float> {
    pub min: F,
    pub max: F,
}

impl<F: Float> PartialEq for Bound<F> {
    fn eq(&self, other: &Self) -> bool {
        self.min == other.min && self.max == other.max
    }
}

impl<F: Float> Eq for Bound<F> {}

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

pub struct GenericTree<'bump, F: Float + Send + Sync, const N: usize, const N2: usize, D: TreeData>
{
    herd: &'bump Herd,
    root: &'bump mut Node<'bump, F, N, N2, D>,
    /// 此树bounds
    bounds: [Bound<F>; N],
    /// 当前节点数量
    pub num: u32,
    /// 子节点间最小的距离
    min_dist: F,
    /// 叶子区域最大的子节点数量 leaf region max children
    leaf_max_children: u32,
}

impl<'bump, F: Float + Send + Sync, const N: usize, const N2: usize, D: TreeData>
    GenericTree<'bump, F, N, N2, D>
{
    pub fn new(
        herd: &'bump Herd,
        bounds: [Bound<F>; N],
        min_dist: F,
        leaf_max_children: u32,
    ) -> Self {
        if leaf_max_children == 0 {
            panic!("leaf_max_children must be greater than 0");
        }

        if (1 << N) != N2 {
            panic!("N2 must be 2^N");
        }

        if leaf_max_children >= N2 as u32 {
            panic!("leaf_max_children cannot >= 2^N")
        }

        GenericTree {
            herd,
            root: herd.get().alloc(Node::new_region(bounds.clone())),
            bounds,
            num: 0,
            min_dist,
            leaf_max_children,
        }
    }

    pub fn add_node(&mut self, node: Node<'bump, F, N, N2, D>) -> Result<(), ()> {
        self.num += 1;
        if !self.root.contains(node.coord()) {
            return Err(());
        }

        let region = self.root.get_leaf_region(node.coord());
        region
            .insert_point(
                self.herd,
                self.herd.get().alloc(node),
                self.leaf_max_children,
            )
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
            self.herd,
            self.herd.get().alloc(Node::new_point(point, data)),
            self.leaf_max_children,
        )
        .unwrap();

        Ok(())
    }

    /// 查询在`max_dist`范围内离point的最近节点
    pub fn find_closest_with_max_dist(
        &self,
        point: &[F; N],
        max_dist: F,
    ) -> Option<&Node<'bump, F, N, N2, D>> {
        let mut stack = vec![&*self.root];
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
                    for child in children.iter().filter(|x| x.is_some()) {
                        let dist = child.as_ref().unwrap().distance(point);
                        if dist < min_dist {
                            stack.push(&**child.as_ref().unwrap());
                        }
                    }
                    ()
                }
            }
        }

        min_ans
    }

    pub fn find_closest(&self, point: &[F; N]) -> Option<&Node<'bump, F, N, N2, D>> {
        self.find_closest_with_max_dist(point, F::infinity())
    }

    /// 使用func: Fn(&Node<F, N, D>, usize)去后序遍历每一个节点
    pub fn visit_post_order_mut<FF>(&mut self, mut func: FF)
    where
        FF: FnMut(&mut Node<'bump, F, N, N2, D>, usize) -> (),
    {
        let mut stack = vec![(self.root as *mut _, 0, true)];
        while let Some((node, depth, is_first)) = stack.pop() {
            let node = unsafe { &mut *node };
            if !is_first {
                func(node, depth);
                continue;
            }

            if node.is_region() {
                stack.push((node, depth, false));
                if let Node::Region { children, .. } = node {
                    for child in children.iter_mut().rev() {
                        if let Some(child) = child {
                            stack.push((*child, depth + 1, true));
                        }
                    }
                }
            } else {
                func(node, depth)
            }
        }
    }

    /// 使用func: Fn(&Node<F, N, D>, usize) -> bool去先序遍历每一个节点
    ///
    /// 如果func返回true，那么该节点的子节点不会被访问
    pub fn visit_pre_order<FF>(&self, func: FF) -> ()
    where
        FF: Fn(&Node<'bump, F, N, N2, D>, usize) -> bool,
    {
        let mut stack = vec![(&self.root, 0)];
        while let Some((node, depth)) = stack.pop() {
            if func(&node, depth) {
                continue;
            }

            match node {
                Node::Point { coord: _, data: _ } => {}
                Node::Region { children, .. } => {
                    for child in children.iter().filter(|x| x.is_some()) {
                        stack.push((&*child.as_ref().unwrap(), depth + 1));
                    }
                }
            }
        }
    }

    /// 使用func: Fn(&Node<F, N, D>, usize) -> bool去先序遍历每一个节点
    ///
    /// 如果func返回true，那么该节点的子节点不会被访问
    pub fn visit_pre_order_mut<FF>(&mut self, mut func: FF) -> ()
    where
        FF: FnMut(&mut Node<'bump, F, N, N2, D>, usize) -> bool,
    {
        let mut stack = vec![(self.root as *mut _, 0)];
        while let Some((node, depth)) = stack.pop() {
            let node = unsafe { &mut *node };
            if func(node, depth) {
                continue;
            }

            match node {
                Node::Point { coord: _, data: _ } => {}
                Node::Region { children, .. } => {
                    for child in children.iter_mut().filter(|x| x.is_some()) {
                        stack.push((*child.as_mut().unwrap(), depth + 1));
                    }
                }
            }
        }
    }
}

impl<'bump, F: Float + Display + Send + Sync, const N: usize, const N2: usize, D: TreeData>
    GenericTree<'bump, F, N, N2, D>
{
    fn debug(&self) {
        let space = String::from(" ");
        self.visit_pre_order(|node, depth| match node {
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

pub trait Distance<F: Float> {
    fn dist(&self, another: &Self) -> F;
}

impl<'bump, F: Float + Sync + Send, const N: usize, const N2: usize, D: TreeData>
    GenericTree<'bump, F, N, N2, D>
{
    pub fn from_nodes(
        herd: &'bump Herd,
        nodes: Vec<Node<'bump, F, N, N2, D>>,
        min_dist: F,
        leaf_max_children: u32,
    ) -> Self {
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

        let mut tree: GenericTree<'bump, F, N, N2, D> =
            GenericTree::new(&herd, bounds, min_dist, leaf_max_children);
        tree.num = nodes.len() as u32;

        let mem = tree.herd.get();
        let mut nodes = nodes.into_iter().map(|x| mem.alloc(x)).collect::<Vec<_>>();

        run(
            tree.herd,
            &mut nodes,
            &mut tree.root,
            leaf_max_children,
            &herd.get(),
        );
        std::mem::forget(nodes);
        return tree;

        fn run<'a, 'bump, F: Float + Send + Sync, const N: usize, const N2: usize, D: TreeData>(
            herd: &'bump Herd,
            nodes: &'a mut [&'bump mut Node<'bump, F, N, N2, D>],
            leaf: &'a mut Node<'bump, F, N, N2, D>,
            leaf_max_children: u32,
            member: &Member<'bump>,
        ) {
            let children_len = {
                match leaf {
                    Node::Point { coord, data } => panic!(),
                    Node::Region {
                        bounds, children, ..
                    } => children.iter().filter(|x| x.is_some()).count(),
                }
            };

            if children_len + nodes.len() <= leaf_max_children as usize {
                for node in nodes {
                    let node: *const _ = &*node;
                    unsafe {
                        leaf.insert_point_directly(std::ptr::read(node));
                    }
                }
            } else {
                let sub_nodes = divide(nodes, leaf.bounds(), N - 1);
                leaf.divide(member).unwrap_or(());
                leaf.children()
                    .into_iter()
                    .filter(|x| x.is_some())
                    .map(|x| &mut **x.as_mut().unwrap())
                    .zip(sub_nodes)
                    .for_each(|(child, nodes)| {
                        run(herd, nodes, &mut *child, leaf_max_children, member)
                    });
            }
        }

        fn divide<
            'b,
            'a,
            'bump,
            F: Float + Send + Sync,
            const N: usize,
            const N2: usize,
            D: TreeData,
        >(
            nodes: &'a mut [&'bump mut Node<'bump, F, N, N2, D>],
            bounds: &'b [Bound<F>; N],
            bound_index: usize,
        ) -> Vec<&'a mut [&'bump mut Node<'bump, F, N, N2, D>]> {
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
        herd: &'bump Herd,
        mut nodes: Vec<&'bump mut Node<'bump, F, N, N2, D>>,
        min_dist: F,
        leaf_max_children: u32,
    ) -> &'bump mut GenericTree<'bump, F, N, N2, D> {
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

        let mut tree: &'bump mut GenericTree<'bump, F, N, N2, D> = herd
            .get()
            .alloc(GenericTree::new(herd, bounds, min_dist, leaf_max_children));
        tree.num = nodes.len() as u32;

        run(
            tree.herd,
            &mut nodes,
            &mut tree.root,
            leaf_max_children,
            0,
            &herd.get(),
        );
        std::mem::forget(nodes);
        return tree;

        fn run<'a, 'bump, F: Float + Send + Sync, const N: usize, const N2: usize, D: TreeData>(
            herd: &'bump Herd,
            nodes: &'a mut [&'bump mut Node<'bump, F, N, N2, D>],
            leaf: &'a mut Node<'bump, F, N, N2, D>,
            leaf_max_children: u32,
            depth: usize,
            member: &Member<'bump>,
        ) {
            debug_assert!(leaf.is_leaf_region());

            if leaf.child_len() + nodes.len() <= leaf_max_children as usize {
                for node in nodes {
                    let node: *const _ = &*node;
                    unsafe {
                        leaf.insert_point_directly(std::ptr::read(node));
                    }
                }
            } else {
                // this is crucial for performance
                if depth <= 2 {
                    let sub_nodes = divide(nodes, leaf.bounds(), N - 1);
                    leaf.divide(member).unwrap_or(());
                    leaf.children()
                        .into_par_iter()
                        .zip(sub_nodes)
                        .for_each(|(child, nodes)| {
                            run(
                                herd,
                                nodes,
                                &mut *child.as_mut().unwrap(),
                                leaf_max_children,
                                depth + 1,
                                &herd.get(),
                            )
                        });
                } else {
                    let sub_nodes = divide(nodes, leaf.bounds(), N - 1);
                    leaf.divide(member).unwrap_or(());
                    leaf.children()
                        .into_iter()
                        .zip(sub_nodes)
                        .for_each(|(child, nodes)| {
                            run(
                                herd,
                                nodes,
                                &mut *child.as_mut().unwrap(),
                                leaf_max_children,
                                depth + 1,
                                member,
                            )
                        });
                }
            }
        }

        fn divide<
            'b,
            'a,
            'bump,
            F: Float + Send + Sync,
            const N: usize,
            const N2: usize,
            D: TreeData,
        >(
            nodes: &'a mut [&'bump mut Node<'bump, F, N, N2, D>],
            bounds: &'b [Bound<F>; N],
            bound_index: usize,
        ) -> Vec<&'a mut [&'bump mut Node<'bump, F, N, N2, D>]> {
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
}

impl<F: Float, const N: usize> Distance<F> for [F; N] {
    //! 计算N维坐标间的距离
    fn dist(&self, another: &Self) -> F {
        let mut square_sum = F::zero();
        for i in 0..N {
            square_sum = square_sum + (self[i] - another[i]).powi(2);
        }

        F::sqrt(square_sum)
    }
}

mod tests {
    use super::{Bound, GenericTree, Node};
    use crate::tree_data::TreeData;
    use bumpalo_herd::Herd;
    use std::thread;

    #[test]
    fn test_post_visit() {
        let herd = Herd::new();
        let mut tree: GenericTree<'_, f64, 2, 4, Data> = GenericTree::new(
            &herd,
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

        for i in 0..100 {
            tree.add([(i) as f64, (i) as f64], i).unwrap();
            let mut visit_order = vec![];
            tree.visit_post_order_mut(|node, _| match node {
                Node::Point { coord, .. } => visit_order.push((Some(coord.clone()), None, 0)),
                Node::Region {
                    bounds, children, ..
                } => visit_order.push((
                    None,
                    Some(bounds.clone()),
                    children.into_iter().filter(|x| x.is_some()).count(),
                )),
            });

            let mut visit_order_2 = vec![];
            tree.root.visit_post_order(&mut |node| match node {
                Node::Point { coord, .. } => {
                    visit_order_2.push((Some(coord.clone()), None, 0));
                }
                Node::Region {
                    bounds, children, ..
                } => {
                    visit_order_2.push((
                        None,
                        Some(bounds.clone()),
                        children.into_iter().filter(|x| x.is_some()).count(),
                    ));
                }
            });

            assert_eq!(visit_order.len(), visit_order_2.len());
            assert_eq!(visit_order, visit_order_2);
        }
    }

    struct Data;
    impl TreeData for Data {
        type PointData = usize;
        type RegionData = usize;
    }

    #[test]
    fn test_debug() {
        let herd = Herd::new();
        let mut tree: GenericTree<'_, f64, 2, 4, Data> = GenericTree::new(
            &herd,
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
        let herd = Herd::new();
        let mut tree: GenericTree<'_, f64, 2, 4, Data> = GenericTree::new(
            &herd,
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
            3,
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
    fn test_from_nodes() {
        let mut nodes = vec![];
        let herd = Herd::new();
        for i in 0..100 {
            for j in 0..100 {
                nodes.push(Node::new_point([i as f64, j as f64], i * 100 + j));
            }
        }

        let tree = GenericTree::<'_, f64, 2, 4, Data>::from_nodes(&herd, nodes, 1.0, 3);
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

    #[test]
    fn test_parallel_inserts() {
        let mut nodes = vec![];
        let herd = Herd::new();
        for i in 0..100 {
            for j in 0..100 {
                nodes.push(
                    herd.get()
                        .alloc(Node::new_point([i as f64, j as f64], i * 100 + j)),
                );
            }
        }

        let tree = GenericTree::<'_, f64, 2, 4, Data>::new_in_par(&herd, nodes, 1.0, 3);
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
