extern crate generic_tree;
extern crate simulation;
extern crate wasm_bindgen;
mod force_graph;
use bumpalo_herd::Herd;
pub use force_graph::ForceGraph2D;
use generic_tree::{parallel, Node, TreeData};
use rayon::prelude::*;
use wasm_bindgen::prelude::*;
pub use wasm_bindgen_rayon::init_thread_pool;

struct Data;
impl TreeData for Data {
    type PointData = usize;
    type RegionData = usize;
}

#[wasm_bindgen]
pub fn build_a_tree(input: &[f32], target: &[f32]) -> usize {
    let herd = Herd::new();
    let mut nodes = vec![];
    let mem = herd.get();
    for i in (0..input.len()).step_by(2) {
        nodes.push(mem.alloc(Node::<'_, f32, 2, 4, Data>::new_point(
            [input[i], input[i + 1]],
            i / 2,
        )));
    }

    let tree = generic_tree::GenericTree::<'_, f32, 2, 4, Data>::new_in_par(&herd, nodes, 0.1, 3);
    // let tree = generic_tree::GenericTree::<f32, 2, usize>::new_in_par(nodes, 0.1, 10);
    let data = *tree.find_closest(&[target[0], target[1]]).unwrap().data();
    data
}
