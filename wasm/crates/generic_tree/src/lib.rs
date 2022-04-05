#![allow(dead_code, unused_imports)]
mod generic_tree;
pub use crate::generic_tree::{Bound, GenericTree, Node};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
