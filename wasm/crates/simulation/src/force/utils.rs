use num::Float;
use rand::prelude::ThreadRng;
use rand::Rng;

pub fn jiggle<F: Float>(rng: &mut ThreadRng) -> F {
    let x = rng.gen_range(0.0..=1.0);
    F::from((x - 0.5) * 1e-6).unwrap()
}

pub fn about_zero<F: Float>(x: F) -> bool {
    x.abs() <= F::epsilon()
}
