use simulation::Simulation;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ForceGraph2D {
    node_positions: Vec<(f64, f64)>,
    simulation: Simulation<f64, 2, ()>,
}

#[wasm_bindgen]
impl ForceGraph2D {
    pub fn get_pos(&self) -> *const (f64, f64) {
        self.node_positions.as_ptr()
    }
}
