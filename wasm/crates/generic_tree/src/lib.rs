#![allow(dead_code, unused_imports)]
mod generic_tree;
mod tree_data;
pub use crate::generic_tree::{Bound, GenericTree, Node};
pub use crate::tree_data::TreeData;
use rayon::{join, prelude::*, ThreadPoolBuilder};

pub fn parallel() {
    for i in 0..10000 {
        rayon::spawn(move || {})
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::parallel;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_parallel() {
        let start = Instant::now();
        parallel();
        println!("{:?}", start.elapsed());
    }
}
