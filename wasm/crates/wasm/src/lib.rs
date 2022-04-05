extern crate generic_tree;
extern crate wasm_bindgen;
use generic_tree::Node;
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
pub fn build_a_tree(input: &[f64], target: &[f64]) -> usize {
    let mut nodes = vec![];
    for i in (0..input.len()).step_by(2) {
        nodes.push(Box::new(Node::<f64, 2, usize>::new_point(
            [input[i], input[i + 1]],
            i / 2,
        )));
    }

    let tree = generic_tree::GenericTree::<f64, 2, usize>::new_in_par(nodes, 0.1, 10);
    *tree.find_closest(&[target[0], target[1]]).unwrap().data()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
