use crate::data::{ForceData, PointData, PointForceData};
use crate::force::utils::{about_zero, jiggle};
use crate::force::ForceSimulate;
use bumpalo_herd::Herd;
use generic_tree::{GenericTree, Node};
use num::Float;
use std::fmt::{Debug, Display, Formatter};

pub struct NBodyForce<F: Float, const N: usize, const N2: usize, D> {
    pub distance_min: F,
    pub distance_max: F,
    pub theta: F,
    pub strength_fn: fn(&[PointData<F, N, D>], usize) -> F,
    pub strengths: Vec<F>,
}

impl<F: Float, const N: usize, const N2: usize, D> Default for NBodyForce<F, N, N2, D> {
    fn default() -> Self {
        NBodyForce {
            distance_min: F::from(0_f64).unwrap(),
            distance_max: F::infinity(),
            theta: F::from(0.9_f64).unwrap(),
            strength_fn: |_, _| F::from(-30_f64).unwrap(),
            strengths: Vec::new(),
        }
    }
}

impl<F: Float, const N: usize, const N2: usize, D> Debug for NBodyForce<F, N, N2, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NBodyForce")
            .field("distance_min", &self.distance_min.to_f64())
            .field("distance_max", &self.distance_max.to_f64())
            .field("theta", &self.theta.to_f64())
            .finish()
    }
}

impl<
        F: Float + Send + Sync,
        const N: usize,
        const N2: usize,
        D: Default + Display + Clone + Send + Sync,
    > NBodyForce<F, N, N2, D>
{
    fn accumulate(&self, node: &mut Node<F, N, N2, ForceData<F, N, D>>) {
        match node {
            Node::Point { data, .. } => {
                data.strength = self.strengths[data.index];
            }
            Node::Region { data, children, .. } => {
                let mut weight = F::zero();
                let mut coord = [F::zero(); N];
                let mut strength = F::zero();
                for child in children.iter_mut().filter(|x| x.is_some()) {
                    let child_node = &*child.as_mut().unwrap();
                    let (_strength, _coord) = match child_node {
                        Node::Point { data, .. } => (data.strength, data.coord),
                        Node::Region { data, .. } => (data.strength, data.coord),
                    };
                    let c = _strength.abs();
                    strength = strength + _strength;
                    weight = weight + c;
                    for i in 0..N {
                        coord[i] = coord[i] + c * _coord[i];
                    }
                }
                for i in 0..N {
                    coord[i] = coord[i] / weight;
                }
                data.coord = coord;
                data.strength = strength;
            }
        }
    }

    fn apply(
        &self,
        point_data: &mut PointData<F, N, D>,
        node: &Node<F, N, N2, ForceData<F, N, D>>,
        alpha: F,
    ) -> bool {
        let n_dim = N as i32;
        let mut rnd = rand::thread_rng();
        // FIXME node的strength 是否会存在未被初始化
        let (_strength, _coord) = match node {
            Node::Point { data, .. } => (data.strength, data.coord),
            Node::Region { data, .. } => (data.strength, data.coord),
        };
        // x维范围
        let w = match node {
            Node::Point { .. } => F::zero(),
            Node::Region { bounds, .. } => bounds[0].width(),
        };
        let mut l = F::zero();
        for i in 0..N {
            l = l + F::powi(_coord[i] - point_data.coord[i], n_dim)
        }
        if F::powi(w / self.theta, n_dim) < l {
            if l < self.distance_max.powi(n_dim) {
                for i in 0..N {
                    if about_zero(_coord[i] - point_data.coord[i]) {
                        let _x: F = jiggle::<F>(&mut rnd);
                        l = l + _x.powi(n_dim)
                    }
                    if l < self.distance_min.powi(n_dim) {
                        let _t: F = self.distance_min.powi(n_dim) * l;
                        l = _t.sqrt()
                    }
                    for j in 0..N {
                        let _d: F = (_coord[i] - point_data.coord[i]) * _strength * alpha / l;
                        point_data.velocity[j] = point_data.velocity[j] + _d;
                    }
                }
            }
            return true;
        } else if node.is_region() || l >= self.distance_max.powi(n_dim) {
            return false;
        }
        // point node
        if point_data.index != node.data().index {
            for i in 0..N {
                if about_zero(_coord[i] - point_data.coord[i]) {
                    let _x: F = jiggle::<F>(&mut rnd);
                    l = l + _x.powi(n_dim)
                }
                if l < self.distance_min.powi(n_dim) {
                    let _t: F = self.distance_min.powi(n_dim) * l;
                    l = _t.sqrt()
                }
            }
            let w = self.strengths[node.data().index] * alpha / l;
            for j in 0..N {
                let _d: F = (_coord[j] - point_data.coord[j]) * w;
                point_data.velocity[j] = point_data.velocity[j] + _d;
            }
        }

        false
    }
}

impl<
        F: Float + Send + Sync,
        const N: usize,
        const N2: usize,
        D: Default + Display + Clone + Send + Sync,
    > ForceSimulate<F, N, D> for NBodyForce<F, N, N2, D>
{
    fn init(&mut self, force_point_data: &[PointData<F, N, D>]) {
        for idx in 0..force_point_data.len() {
            self.strengths
                .push((self.strength_fn)(force_point_data, idx))
        }
    }

    fn force(&self, force_point_data: &mut [PointData<F, N, D>], alpha: F) {
        // for point_data in force_point_data.iter() {
        //     println!("更新前数据 {}", point_data)
        // }
        // TODO 效率
        let herd = Herd::new();
        let tree = GenericTree::<F, N, N2, ForceData<F, N, D>>::new_in_par(
            &herd,
            force_point_data
                .iter_mut()
                .map(|point_data| {
                    herd.get().alloc(Node::new_point(
                        point_data.coord,
                        PointForceData::from_point_data(point_data),
                    ))
                })
                .collect::<Vec<_>>(),
            // TODO 参数设置
            F::infinity(),
            1, //(N.pow(2_u32) - 1) as u32,
        );
        tree.visit_post_order_mut(|node, _| self.accumulate(node));
        for point_data in force_point_data.iter_mut() {
            tree.visit_pre_order_mut(|node, _| self.apply(point_data, node, alpha));
        }
        // for point_data in force_point_data.iter() {
        //     println!("更新后数据 {}", point_data)
        // }
    }
}
