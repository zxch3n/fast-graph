use std::fmt::{Display, Formatter};

use simulation::{
    force::{CenterForce, LinkForce, NBodyForce, PositionForce},
    Simulation,
};
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
    node_positions: Vec<(f32, f32)>,
    simulation: Simulation<f32, 2, RandomData>,
}

#[wasm_bindgen]
impl ForceGraph2D {
    // pub fn from_random(node_num: usize) -> Self {
    //     let mut data = Vec::with_capacity(node_num);
    //     for _ in 0..node_num {
    //         data.push(RandomData::default())
    //     }
    //     let node_positions = vec![(0., 0.); node_num];
    //     let mut simulation: Simulation<f32, 2, RandomData> = Simulation::from_data(data);

    //     let mut out = ForceGraph2D {
    //         node_positions,
    //         simulation,
    //     };

    //     out.tick(1, false);
    //     out
    // }

    pub fn build_graph(node_num: usize, links_data: &[usize]) -> Self {
        let mut data = Vec::with_capacity(node_num);
        for _ in 0..node_num {
            data.push(RandomData::default())
        }
        let node_positions = vec![(0., 0.); node_num];
        let mut simulation: Simulation<f32, 2, RandomData> = Simulation::from_data(data);
        let mut link_force = LinkForce::default();
        let mut links = Vec::new();
        for i in (0..links_data.len()).step_by(2) {
            links.push((links_data[i], links_data[i + 1]));
        }
        link_force.set_links(links);
        simulation.add_force(String::from("link"), Box::new(link_force));

        let mut out = ForceGraph2D {
            node_positions,
            simulation,
        };

        out.tick(1, false);
        out
    }

    pub fn add_n_body_force(&mut self) {
        let mut nbody_force: NBodyForce<f32, 2, 4, RandomData> = NBodyForce::default();
        nbody_force.distance_min = 10_f32;
        nbody_force.set_strength_fn(|_, _| -1_f32);
        self.simulation
            .add_force(String::from("official:n-body"), Box::new(nbody_force));
    }

    pub fn add_center_force(&mut self) {
        self.simulation.add_force(
            String::from("official:center-force"),
            Box::new(CenterForce::default()),
        );
    }

    pub fn tick(&mut self, times: usize, changed: bool) {
        if changed {
            for (i, point) in self.simulation.force_point_data.iter_mut().enumerate() {
                point.coord[0] = self.node_positions[i].0;
                point.coord[1] = self.node_positions[i].1;
            }
        }

        for _ in 0..times {
            self.simulation.tick();
        }

        for (i, point) in self.simulation.force_point_data.iter().enumerate() {
            self.node_positions[i] = (point.coord[0], point.coord[1]);
        }
    }

    pub fn get_pos(&self) -> *const (f32, f32) {
        self.node_positions.as_ptr()
    }
}
