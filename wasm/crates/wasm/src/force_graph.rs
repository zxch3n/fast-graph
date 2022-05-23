use std::fmt::{Display, Formatter};

use simulation::{force::NBodyForce, Simulation};
use wasm_bindgen::prelude::*;

#[derive(Clone)]
struct RandomData {
    data: Vec<i32>,
}

impl Default for RandomData {
    fn default() -> Self {
        RandomData { data: vec![] }
    }
}

impl Display for RandomData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

#[wasm_bindgen]
pub struct ForceGraph2D {
    node_positions: Vec<(f64, f64)>,
    simulation: Simulation<f64, 2, RandomData>,
}

#[wasm_bindgen]
impl ForceGraph2D {
    pub fn from_random(node_num: usize) -> Self {
        let mut data = Vec::with_capacity(node_num);
        for _ in 0..node_num {
            data.push(RandomData::default())
        }
        let mut node_positions = vec![(0., 0.); node_num];
        let mut simulation: Simulation<f64, 2, RandomData> = Simulation::from_data(data);
        simulation.add_force(
            String::from("n-body"),
            Box::new(NBodyForce::<f64, 2, 4, RandomData>::default()),
        );

        for (i, point) in simulation.force_point_data.iter().enumerate() {
            node_positions[i] = (point.coord[0], point.coord[1]);
        }

        ForceGraph2D {
            node_positions,
            simulation,
        }
    }

    pub fn tick(&mut self, times: usize) {
        for _ in 0..times {
            self.simulation.tick();
        }

        for (i, point) in self.simulation.force_point_data.iter().enumerate() {
            self.node_positions[i] = (point.coord[0], point.coord[1]);
        }
    }

    pub fn get_pos(&self) -> *const (f64, f64) {
        self.node_positions.as_ptr()
    }
}
