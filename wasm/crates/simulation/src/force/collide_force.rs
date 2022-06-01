use crate::data::{ForceData, PointData, PointForceData};
use crate::force::utils::{about_zero, jiggle, print_node_data};
use crate::force::ForceSimulate;
use bumpalo_herd::Herd;
use generic_tree::{GenericTree, Node};
use num::Float;
use std::fmt::Display;

pub struct CollideForce<F: Float, const N: usize, const N2: usize, D> {
    pub radius: Vec<F>,
    pub strength: F,
    pub iterations: usize,
    force_point_data: Option<*const [PointData<F, N, D>]>,
}

impl<
        F: Float + Send + Sync,
        const N: usize,
        const N2: usize,
        D: Default + Display + Clone + Send + Sync,
    > CollideForce<F, N, N2, D>
{
    pub fn new(radius: Vec<F>, strength: F, iterations: usize) -> CollideForce<F, N, N2, D> {
        CollideForce {
            radius,
            strength,
            iterations,
            force_point_data: None,
        }
    }

    fn prepare(&self, node: &mut Node<F, N, N2, ForceData<F, N, D>>) {
        match node {
            Node::Point { data, .. } => {
                data.radius = self.radius[data.index];
            }
            Node::Region { data, children, .. } => {
                let mut r_radius = match data.radius {
                    Some(rr) => rr,
                    None => F::zero(),
                };
                for child in children.iter_mut().filter(|x| x.is_some()) {
                    let child_node = &*child.as_mut().unwrap();
                    match child_node {
                        Node::Point { data: p_data, .. } => {
                            if p_data.radius > r_radius {
                                r_radius = p_data.radius
                            }
                        }
                        Node::Region { data: r_data, .. } => {
                            if let Some(rr_radius) = r_data.radius {
                                if rr_radius > r_radius {
                                    r_radius = rr_radius
                                }
                            }
                        }
                    };
                }
                data.radius = Some(r_radius);
            }
        }
    }

    fn apply(
        &self,
        point_data: &mut PointData<F, N, D>,
        node: &Node<F, N, N2, ForceData<F, N, D>>,
    ) -> bool {
        if node.is_region() && !node.has_children() {
            // 跳过无children的Region
            return true;
        }
        let mut rnd = rand::thread_rng();
        let ri = self.radius[point_data.index];
        let ri2 = ri * ri;
        let mut rj = match node {
            Node::Point { data, .. } => data.radius,
            Node::Region { data, .. } => data.radius.unwrap(),
        };
        let mut r = ri + rj;
        let mut cn = [F::zero(); N];
        for i in 0..N {
            cn[i] = cn[i] + point_data.coord[i] + point_data.velocity[i]
        }
        if !node.is_region() {
            // Point
            let data: &PointData<F, N, D> = node.data();
            if data.index > point_data.index {
                let mut p = [F::zero(); N];
                for i in 0..N {
                    p[i] = p[i] + cn[i] - data.coord[i] - data.velocity[i]
                }
                let mut l = p.iter().fold(F::zero(), |s, &x| s + x * x);
                if l < r * r {
                    for i in 0..N {
                        if about_zero(p[i]) {
                            p[i] = jiggle::<F>(&mut rnd);
                            l = l + p[i] * p[i];
                        }
                    }
                    l = l.sqrt();
                    l = (r - l) / l * self.strength;
                    rj = rj * rj;
                    r = rj / (ri2 + rj);
                    for i in 0..N {
                        p[0] = p[0] * l;
                        point_data.velocity[i] = point_data.velocity[i] + r;
                    }
                    r = F::one() - r;
                    for i in 0..N {
                        point_data.velocity[i] = point_data.velocity[i] - p[i] * r;
                    }
                }
            }
            return false;
        }

        (0..N).fold(false, |b, i| {
            b || (node.bounds()[i].min > cn[i] + r) || (node.bounds()[i].max < cn[i] - r)
        })
    }
}

impl<
        F: Float + Send + Sync,
        const N: usize,
        const N2: usize,
        D: Default + Display + Clone + Send + Sync,
    > ForceSimulate<F, N, D> for CollideForce<F, N, N2, D>
{
    fn init(&mut self, force_point_data: &[PointData<F, N, D>]) {
        self.force_point_data = Some(force_point_data as *const [PointData<F, N, D>]);
    }

    fn force(&self, force_point_data: &mut [PointData<F, N, D>], alpha: F) {
        for _ in 0..self.iterations {
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

            tree.visit_post_order_mut(|node, _| self.prepare(node));

            for point_data in force_point_data.iter_mut() {
                tree.visit_pre_order_mut(|node, _| self.apply(point_data, node));
            }
        }
    }
}
