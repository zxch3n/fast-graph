use std::fmt::Display;

use num::Float;

#[derive(Clone, Copy, Debug)]
pub struct Bound<F: Float> {
    min: F,
    max: F,
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

#[derive(Clone, Debug)]
pub enum Node<F: Float, const N: usize, D> {
    Point {
        coord: [F; N],
        data: D,
    },
    Region {
        bounds: [Bound<F>; N],
        children: Vec<Box<Node<F, N, D>>>,
    },
}

#[inline]
fn two_power(n: usize) -> usize {
    1 << n
}

impl<F: Float, const N: usize, D> Node<F, N, D> {
    pub fn new_region(bounds: [Bound<F>; N]) -> Self {
        Node::Region {
            bounds,
            children: vec![],
        }
    }

    pub fn new_point(coord: [F; N], data: D) -> Self {
        Node::Point { coord, data }
    }

    pub fn is_region(&self) -> bool {
        match self {
            Node::Point { coord: _, data: _ } => false,
            Node::Region {
                bounds: _,
                children: _,
            } => true,
        }
    }

    pub fn distance(&self, point: &[F; N]) -> F {
        if self.contains(point) {
            return F::zero();
        }

        match self {
            Node::Point { coord, data } => coord.dist(point),
            Node::Region { bounds, children } => {
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
            Node::Region {
                bounds: _,
                children,
            } => {
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
            Node::Region { bounds, children } => {
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
            Node::Region {
                bounds,
                children: _,
            } => {
                for i in 0..N {
                    if point[i] < bounds[i].min || point[i] > bounds[i].max {
                        return false;
                    }
                }

                true
            }
        }
    }

    pub fn get_sub_region(&self, point: &[F; N]) -> usize {
        match self {
            Node::Region {
                bounds,
                children: _,
            } => {
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
                Node::Region {
                    bounds: _,
                    children,
                } => {
                    node = &mut children[index];
                }
                _ => panic!(),
            }
        }

        node
    }

    fn insert_point(&mut self, point: Box<Self>, max_num: usize) -> Result<(), ()> {
        if !self.is_leaf_region() {
            return Err(());
        }

        let mut should_divide = false;
        if !self.contains(point.coord()) {
            panic!();
        }

        match self {
            Node::Point { coord: _, data: _ } => return Err(()),
            Node::Region {
                bounds: _,
                children,
            } => {
                children.push(point);
                if children.len() > max_num {
                    should_divide = true;
                }
            }
        }

        if should_divide {
            let mut points = vec![];
            if let Node::Region {
                bounds: _,
                children,
            } = self
            {
                while let Some(node) = children.pop() {
                    points.push(node);
                }
            }

            self.divide().unwrap_or(());
            for point in points {
                match &*point {
                    Node::Point { coord, data: _ } => {
                        let node = Some(self.get_leaf_region(&coord));
                        node.unwrap().insert_point(point, max_num);
                    }
                    Node::Region {
                        bounds: _,
                        children: _,
                    } => panic!(),
                }
            }
        }

        Ok(())
    }

    pub fn coord(&self) -> &[F; N] {
        match self {
            Node::Point { coord, data: _ } => coord,
            _ => panic!(),
        }
    }

    pub fn data(&self) -> &D {
        match self {
            Node::Point { coord: _, data } => data,
            _ => panic!(),
        }
    }

    #[cfg(debug_assertions)]
    fn check(&self) -> Result<(), ()> {
        match self {
            Node::Point { coord: _, data: _ } => Ok(()),
            Node::Region { bounds, children } => {
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
}

pub struct GenericTree<F: Float, const N: usize, D> {
    root: Node<F, N, D>,
    bounds: [Bound<F>; N],
    num: u32,
    min_dist: F,
    /**
     * leaf region max children
     */
    leaf_max_children: u32,
}

impl<F: Float, const N: usize, D> GenericTree<F, N, D> {
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

    pub fn add(&mut self, point: [F; N], data: D) -> Result<(), ()> {
        self.num += 1;
        if !self.root.contains(&point) {
            return Err(());
        }

        let node = self.root.get_leaf_region(&point);
        node.insert_point(
            Box::new(Node::new_point(point, data)),
            self.leaf_max_children as usize,
        )
        .unwrap();

        Ok(())
    }

    pub fn find_closest(&mut self, point: &[F; N]) -> Option<&Node<F, N, D>> {
        let mut stack = vec![&self.root];
        let mut min_dist = F::infinity();
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
                Node::Region {
                    bounds: _,
                    children,
                } => {
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
                Node::Region {
                    bounds: _,
                    children,
                } => {
                    for child in children {
                        stack.push((child, depth + 1));
                    }
                }
            }
        }
    }
}

impl<F: Float + Display, const N: usize, D: Display> GenericTree<F, N, D> {
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
            Node::Region { bounds, children } => {
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

impl<F: Float, const N: usize> Distance<F> for [F; N] {
    fn dist(&self, another: &Self) -> F {
        let mut square_sum = F::zero();
        for i in 0..N {
            square_sum = square_sum + (self[i] - another[i]).powi(2);
        }

        F::sqrt(square_sum)
    }
}

#[cfg(test)]
mod tests {
    use super::{Bound, GenericTree};
    #[test]
    fn test_debug() {
        let mut tree: GenericTree<f64, 2, usize> = GenericTree::new(
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
        let mut tree: GenericTree<f64, 2, usize> = GenericTree::new(
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
}
