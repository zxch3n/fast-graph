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

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    unsafe {
        alert(&format!("Hello, {}!", name));
    }
}

#[wasm_bindgen]
pub fn sum_of_squares(input: &[i32]) -> i32 {
    input.par_iter().map(|i| i * i).sum()
}

#[wasm_bindgen]
pub fn js_parallel() {
    parallel();
}

struct Data;
impl TreeData for Data {
    type PointData = usize;
    type RegionData = usize;
}

#[wasm_bindgen]
pub fn build_a_tree(input: &[f64], target: &[f64]) -> usize {
    let herd = Herd::new();
    let mut nodes = vec![];
    let mem = herd.get();
    for i in (0..input.len()).step_by(2) {
        nodes.push(mem.alloc(Node::<'_, f64, 2, 4, Data>::new_point(
            [input[i], input[i + 1]],
            i / 2,
        )));
    }

    let tree = generic_tree::GenericTree::<'_, f64, 2, 4, Data>::new_in_par(&herd, nodes, 0.1, 3);
    // let tree = generic_tree::GenericTree::<f64, 2, usize>::new_in_par(nodes, 0.1, 10);
    let data = *tree.find_closest(&[target[0], target[1]]).unwrap().data();
    data
}

/// 这种类型的并行在 WASM 上优化效果很好
#[wasm_bindgen]
pub fn heavy_calc(parallel: bool) -> usize {
    if parallel {
        (0..1_000_000)
            .collect::<Vec<usize>>()
            .par_iter()
            .map(|&i| {
                let mut sum = 0;
                for m in (i..i + 100) {
                    sum += sum - i * i;
                }
                sum
            })
            .max()
            .unwrap()
    } else {
        (0..1_000_000)
            .collect::<Vec<usize>>()
            .iter()
            .map(|&i| {
                let mut sum = 0;
                for m in (i..i + 100) {
                    sum += sum - i * i;
                }
                sum
            })
            .max()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
