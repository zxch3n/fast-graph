use crate::data::{ForceData, PointData};
use generic_tree::Node;
use num::Float;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::fmt::Display;

pub fn jiggle<F: Float>(rng: &mut ThreadRng) -> F {
    let x = rng.gen_range(0.0..=1.0);
    F::from((x - 0.5) * 1e-6).unwrap()
}

pub fn about_zero<F: Float>(x: F) -> bool {
    x.abs() <= F::epsilon()
}

pub fn print_node_data<
    F: Float + Send + Sync,
    const N: usize,
    const N2: usize,
    D: Clone + Send + Sync + Default + Display,
>(
    node: &Node<F, N, N2, ForceData<F, N, D>>,
) {
    match node {
        Node::Point { data, .. } => {
            println!("{}", data as &PointData<F, N, D>)
        }
        Node::Region { data, .. } => {
            println!("{}", data)
        }
    };
}
