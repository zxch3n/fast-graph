use crate::force_data::PointForceData;
use lazy_static::lazy_static;
use num::Float;
use std::collections::HashMap;
use std::f64::consts::PI;

lazy_static! {
    static ref INITIAL_RADIUS: f64 = 10.0;
    static ref INITIAL_ANGLE_ROLL: f64 = PI * (3.0 - 5.0.sqrt()); // Golden ratio angle
    static ref INITIAL_ANGLE_YAW: f64 = PI * 20.0 / (9.0 + 221.0.sqrt()); // Markov irrational number
}

/// 物理模拟
pub struct Simulation<F: Float, const N: usize, D> {
    /// 真实数据列表，用以初始化，初始化后将被封装在ForceNode节点中持有引用
    pub force_point_data: Vec<PointForceData<F, N, D>>,
    /// 全部物理力
    pub forces: HashMap<String, Box<dyn ForceSimulate<F, N, D>>>,
    /// 每一时间刻，会降低alpha（根据alpha_decay和alpha_target），当alpha小于alpha_min，将停止物理模拟
    alpha: F,
    alpha_min: F,
    alpha_decay: F,
    alpha_target: F,
    /// 每一时间刻，节点速度下降率
    velocity_decay: F,
}

pub trait ForceSimulate<F: Float, const N: usize, D> {
    fn init(&mut self, force_point_data: &[PointForceData<F, N, D>]);
    fn force(&self, force_point_data: &mut [PointForceData<F, N, D>], alpha: F);
}

impl<F: Float, const N: usize, D> Default for Simulation<F, N, D> {
    fn default() -> Self {
        Simulation {
            force_point_data: Vec::new(),
            forces: HashMap::new(),
            alpha: F::one(),
            alpha_min: F::from(0.001).unwrap(),
            alpha_decay: F::from(1.0 - 0.001.powf(1.0 / 300.0)).unwrap(),
            alpha_target: F::zero(),
            velocity_decay: F::from(0.6).unwrap(),
        }
    }
}

impl<F: Float, const N: usize, D> Simulation<F, N, D> {
    pub fn from_data(data: Vec<D>) -> Simulation<F, N, D> {
        let mut simulation = Simulation::default();
        let nodes = Self::init_point_data(data);
        simulation.force_point_data.extend(nodes);
        simulation
    }

    /// Initialize [Simulation.force_point_data] through data
    /// if Simulation initialized by [Simulation::default()]
    pub fn set_data(&mut self, data: Vec<D>) {
        let nodes = Self::init_point_data(data);
        self.force_point_data.extend(nodes);
    }

    pub fn add_force(&mut self, name: String, mut force: Box<dyn ForceSimulate<F, N, D>>) {
        force.init(&self.force_point_data);
        self.forces.insert(name, force);
    }

    pub fn remove_force(&mut self, name: &str) -> Option<Box<dyn ForceSimulate<F, N, D>>> {
        self.forces.remove(name)
    }

    pub fn tick(&mut self) {
        self.alpha = self.alpha + (self.alpha_target - self.alpha) * self.alpha_decay;

        for (_, force) in &self.forces {
            force.force(&mut self.force_point_data, self.alpha)
        }

        for point_data in &mut self.force_point_data {
            match point_data.fixed_position {
                None => {
                    for i in 0..N {
                        point_data.velocity[i] = point_data.velocity[i] * self.velocity_decay;
                    }
                    let v = point_data.velocity;
                    let coord = point_data.coord_mut();
                    for i in 0..N {
                        coord[i] = coord[i] + v[i];
                    }
                }
                Some(fixed_position) => {
                    *point_data.coord_mut() = fixed_position;
                    point_data.velocity = [F::zero(); N]
                }
            }
        }
    }

    fn init_point_data(data: Vec<D>) -> Vec<PointForceData<F, N, D>> {
        let mut nodes = Vec::with_capacity(data.len());
        for (idx, datum) in data.into_iter().enumerate() {
            let idx_f = idx as f64;
            let radius = *INITIAL_RADIUS
                * match N {
                    1 => idx_f,
                    2 => (0.5 + idx_f).sqrt(),
                    3 => (0.5 + idx_f).cbrt(),
                    _ => panic!("unsupported dim > 3 "),
                };
            let roll_angle = idx_f * *INITIAL_ANGLE_ROLL;
            let yaw_angle = idx_f * *INITIAL_ANGLE_YAW;
            let mut coord: [F; N] = [F::zero(); N];
            match N {
                1 => coord[0] = F::from(radius).unwrap(),
                2 => {
                    coord[0] = F::from(radius * roll_angle.cos()).unwrap();
                    coord[1] = F::from(radius * roll_angle.sin()).unwrap()
                }
                3 => {
                    coord[0] = F::from(radius * roll_angle.sin() * yaw_angle.cos()).unwrap();
                    coord[1] = F::from(radius * roll_angle.cos()).unwrap();
                    coord[2] = F::from(radius * roll_angle.sin() * yaw_angle.sin()).unwrap();
                }
                _ => panic!("unsupported dim > 3 "),
            };
            nodes.push(PointForceData::from_data(datum, coord, idx))
        }
        nodes
    }
}

mod tests {
    use crate::{force::NBodyForce, simulation::Simulation};
    use generic_tree::Node;
    use std::time::Instant;

    #[test]
    fn test_simulation_init() {
        let mut simulation: Simulation<f64, 2, i32> = Simulation::from_data(vec![1, 2, 3]);
        for (i, force_node) in simulation.force_point_data.iter().enumerate() {
            let coord = &force_node.coord;
            match i {
                0 => {
                    assert!((coord[0] - 7.0710678118654755).abs() < f64::EPSILON);
                    assert!((coord[1] - 0.0).abs() < f64::EPSILON)
                }
                1 => {
                    assert!((coord[0] - -9.03088751750192).abs() < f64::EPSILON);
                    assert!((coord[1] - 8.273032735715967).abs() < f64::EPSILON)
                }
                2 => {
                    assert!((coord[0] - 1.3823220809823638).abs() < f64::EPSILON);
                    assert!((coord[1] - -15.750847141167634).abs() < f64::EPSILON)
                }
                _ => panic!(),
            }
        }
    }

    #[test]
    fn test_nbody_force() {
        let mut simulation: Simulation<f64, 2, i32> = Simulation::from_data(vec![1, 2, 3]);
        let nbody_force = NBodyForce::<f64, 2, 4, i32>::default();
        simulation.add_force(String::from("n-body"), Box::new(nbody_force));
        for (index, data) in simulation.force_point_data.iter().enumerate() {
            let v = data.velocity;
            match index {
                0 => {
                    assert!((v[0] - 1.3447204982634113).abs() < f64::EPSILON);
                    assert!((v[1] - 1.3447204982634113).abs() < f64::EPSILON)
                }
                1 => {
                    assert!((v[0] - 0.34921490951356354).abs() < f64::EPSILON);
                    assert!((v[1] - 0.34921490951356354).abs() < f64::EPSILON)
                }
                2 => {
                    assert!((v[0] - -0.34921490951356354).abs() < f64::EPSILON);
                    assert!((v[1] - -0.34921490951356354).abs() < f64::EPSILON)
                }
                _ => panic!(),
            }
        }
    }

    #[test]
    fn test_tick() {
        for node_num in [100, 1000, 10000, 100000] {
            let mut simulation: Simulation<f64, 2, i32> =
                Simulation::from_data(Vec::from_iter(0..node_num));
            simulation.add_force(
                String::from("n-body"),
                Box::new(NBodyForce::<f64, 2, 4, i32>::default()),
            );
            let start = Instant::now();
            simulation.tick();
            println!(
                "{} node, a tick time: {} ms",
                node_num,
                start.elapsed().as_millis()
            );
        }
    }
}
