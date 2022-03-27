use num::Float;

#[derive(Clone, Copy, Debug)]
pub struct Bound<F: Float> {
    min: F,
    max: F,
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
                    let mut masks = vec![];
                    let mut mask = 1 << (N - 1);
                    for _ in 0..N {
                        children_bounds.push(bounds.clone());
                        masks.push(mask);
                        mask >>= 1;
                    }

                    for i in 0..N {
                        for j in 0..N {
                            if (i & masks[j]) > 0 {
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
                for i in 0..N {
                    let m = (bounds[i].min + bounds[i].max) / F::from(2.0).unwrap();
                    if point[i] > m {
                        index += 1;
                    }
                    index <<= 1;
                }

                index
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
                let mut node = None;
                match &*point {
                    Node::Point { coord, data } => {
                        node = Some(self.get_leaf_region(&coord));
                    }
                    Node::Region { bounds, children } => panic!(),
                }

                node.unwrap().insert_point(point, max_num);
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
    fn test_add() {
        let mut tree: GenericTree<f64, 2, ()> = GenericTree::new(
            [
                Bound {
                    min: 0.0,
                    max: 1024.0,
                },
                Bound {
                    min: 0.0,
                    max: 1024.0,
                },
            ],
            0.1,
            10,
        );

        tree.add([0.0, 0.0], ()).unwrap();
        tree.add([512.0, 512.0], ()).unwrap();

        let a = tree.find_closest(&[2.0, 2.0]).unwrap();
        assert_eq!(a.coord(), &[0.0, 0.0]);
    }
}
