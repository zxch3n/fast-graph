use crate::data::{ForceData, PointData, PointForceData};
use crate::force::utils::{about_zero, jiggle};
use crate::force::ForceSimulate;
use bumpalo_herd::Herd;
use generic_tree::{GenericTree, Node, TreeData};
use num::Float;
use std::fmt::{Debug, Display, Formatter};

pub struct NBodyForce<F: Float, const N: usize, const N2: usize, D> {
    pub distance_min: F,
    pub distance_max: F,
    pub theta: F,
    pub strength_fn: fn(&PointData<F, N, D>, &[PointData<F, N, D>]) -> F,
    strengths: Vec<F>,
    force_point_data: Option<*const [PointData<F, N, D>]>,
}

impl<F: Float, const N: usize, const N2: usize, D> Default for NBodyForce<F, N, N2, D> {
    fn default() -> Self {
        NBodyForce {
            distance_min: F::from(0_f64).unwrap(),
            distance_max: F::infinity(),
            theta: F::from(0.9_f64).unwrap(),
            strength_fn: |_, _| F::from(-30_f64).unwrap(),
            strengths: Vec::new(),
            force_point_data: None,
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
    pub fn new(
        distance_min: F,
        distance_max: F,
        theta: F,
        strength_fn: fn(&PointData<F, N, D>, &[PointData<F, N, D>]) -> F,
    ) -> NBodyForce<F, N, N2, D> {
        NBodyForce {
            distance_min,
            distance_max,
            theta,
            strength_fn,
            strengths: Vec::new(),
            force_point_data: None,
        }
    }

    pub fn set_strength_fn(
        &mut self,
        strength_fn: fn(&PointData<F, N, D>, &[PointData<F, N, D>]) -> F,
    ) {
        self.strength_fn = strength_fn;
        if let Some(force_point_data) = self.force_point_data {
            unsafe { self.init(force_point_data.as_ref().unwrap()) }
        }
    }

    fn accumulate(&self, node: &mut Node<F, N, N2, ForceData<F, N, D>>) {
        if node.is_region() && !node.has_children() {
            return;
        }
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
                        Node::Point { data, .. } => (Some(data.strength), Some(data.coord)),
                        Node::Region { data, .. } => (data.strength, data.coord),
                    };
                    if let (Some(_strength), Some(_coord)) = (_strength, _coord) {
                        let c = _strength.abs();
                        strength = strength + _strength;
                        weight = weight + c;
                        for i in 0..N {
                            coord[i] = coord[i] + c * _coord[i];
                        }
                    }
                }
                for i in 0..N {
                    coord[i] = coord[i] / weight;
                }
                (data.coord, data.strength) = match about_zero(weight) {
                    false => (Some(coord), Some(strength)),
                    true => (None, None),
                }
            }
        }
    }

    fn apply(
        &self,
        point_data: &mut PointData<F, N, D>,
        node: &Node<F, N, N2, ForceData<F, N, D>>,
        alpha: F,
    ) -> bool {
        if node.is_region() && !node.has_children() {
            // 跳过无children的Region
            return true;
        }
        let mut rnd = rand::thread_rng();
        // FIXME node的strength 是否会存在未被初始化
        let (_strength, _coord) = match node {
            Node::Point { data, .. } => (Some(data.strength), Some(data.coord)),
            Node::Region { data, .. } => (data.strength, data.coord),
        };

        if let (Some(_strength), Some(_coord)) = (_strength, _coord) {
            // x维范围
            let w = match node {
                Node::Point { .. } => F::zero(),
                Node::Region { bounds, .. } => bounds[0].width(),
            };
            let mut l = F::zero();
            for i in 0..N {
                l = l + F::powi(_coord[i] - point_data.coord[i], 2)
            }
            if F::powi(w / self.theta, 2) < l {
                if l < self.distance_max.powi(2) {
                    for i in 0..N {
                        if about_zero(_coord[i] - point_data.coord[i]) {
                            let _x: F = jiggle::<F>(&mut rnd);
                            l = l + _x.powi(2)
                        }
                        if l < self.distance_min.powi(2) {
                            let _t: F = self.distance_min.powi(2) * l;
                            l = _t.sqrt()
                        }
                        for j in 0..N {
                            let _d: F = (_coord[j] - point_data.coord[j]) * _strength * alpha / l;
                            point_data.velocity[j] = point_data.velocity[j] + _d;
                        }
                    }
                }
                return true;
            } else if node.is_region() || l >= self.distance_max.powi(2) {
                return false;
            }
            // point node
            if point_data.index != node.data().index {
                for i in 0..N {
                    if about_zero(_coord[i] - point_data.coord[i]) {
                        let _x: F = jiggle::<F>(&mut rnd);
                        l = l + _x.powi(2)
                    }
                    if l < self.distance_min.powi(2) {
                        let _t: F = self.distance_min.powi(2) * l;
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
        } else {
            // 应该不会进入这里
            true
        }
    }

    fn _set_strength(&mut self) {
        if let Some(force_point_data) = self.force_point_data {
            unsafe {
                (&*force_point_data).iter().for_each(|point_data| {
                    self.strengths[point_data.index] =
                        (self.strength_fn)(point_data, &*force_point_data)
                })
            }
        }
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
        self.force_point_data = Some(force_point_data as *const [PointData<F, N, D>]);
        self.strengths = vec![F::zero(); force_point_data.len()];
        self._set_strength()
    }

    fn force(&self, force_point_data: &mut [PointData<F, N, D>], alpha: F) {
        // for point_data in force_point_data.iter() {
        //     println!("更新前数据 {}", point_data)
        // }
        // TODO 效率
        let herd = Herd::new();
        let mut tree = GenericTree::<F, N, N2, ForceData<F, N, D>>::from_nodes(
            &herd,
            force_point_data
                .iter_mut()
                .map(|point_data| {
                    Node::new_point(
                        point_data.coord,
                        PointForceData::from_point_data(point_data),
                    )
                })
                .collect::<Vec<_>>(),
            // TODO 参数设置
            F::infinity(),
            (N.pow(2_u32) - 1) as u32,
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
