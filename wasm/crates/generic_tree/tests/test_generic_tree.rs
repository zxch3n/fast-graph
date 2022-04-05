#![feature(test)]

extern crate test;
use std::time::Instant;

use generic_tree::{Bound, GenericTree, Node};
use rand::Rng;
use rayon::ThreadPoolBuilder;
use test::{black_box, Bencher};
extern crate rand;

#[bench]
fn test_parallel_inserts(bench: &mut Bencher) {
    // M1
    // 1 threads 151.22222222222223ms
    // 2 threads 87.72222222222223ms
    // 4 threads 54.333333333333336ms
    // 8 threads 45.5ms

    // AMD Ryzen 9 3900X 12-Core Processor 3.80 GHz
    // 1 threads 230.83333333333334ms
    // 2 threads 129.22222222222223ms
    // 4 threads 79.05555555555556ms
    // 8 threads 55.94444444444444ms
    // 16 threads 48.22222222222222ms
    // 24 threads 51.94444444444444ms

    for thread_num in [1, 2, 4, 8] {
        let pool = ThreadPoolBuilder::new()
            .num_threads(thread_num)
            .build()
            .unwrap();
        pool.install(|| {
            let mut rng = rand::thread_rng();

            // let start = Instant::now();
            let mut nodes = vec![];
            for i in 0..1000 {
                for j in 0..1000 {
                    nodes.push(Box::new(Node::new_point(
                        [(rng.gen::<f64>()) * 1000., rng.gen::<f64>() * 1000.],
                        i * 1000 + j,
                    )));
                }
            }

            let mut durations = vec![];
            let mut temp = vec![];
            for _ in 0..20 {
                let nodes = nodes.clone();
                let start = Instant::now();
                let tree = GenericTree::<f64, 2, usize>::new_in_par(nodes, 1.0, 10);
                let duration = start.elapsed().as_millis();
                durations.push(duration);
                // let start = Instant::now();
                temp.push(tree.num);
                // DROP IN ANOTHER THREAD! IMPORTANT!
                pool.spawn(move || drop(tree));
                // println!("drop {}ms", start.elapsed().as_millis());
            }

            durations.sort();
            let slice = &durations[1..durations.len() - 1];
            let avg = (slice.iter().sum::<u128>() as f64) / (slice.len() as f64);
            println!("{} threads {}ms", thread_num, avg);
        })
    }
}

#[bench]
fn bench_single_thread_inserts(bench: &mut Bencher) {
    // 579ms on random data / 167ms if data is evenly distributed

    bench.iter(black_box(|| {
        let mut nodes = vec![];
        let mut rng = rand::thread_rng();
        for i in 0..1000 {
            for j in 0..1000 {
                nodes.push(Node::new_point(
                    [(rng.gen::<f64>()) * 1000., rng.gen::<f64>() * 1000.],
                    i * 1000 + j,
                ));
            }
        }
        let mut tree = GenericTree::<f64, 2, usize>::new(
            [
                Bound {
                    min: -1.0,
                    max: 1001.0,
                },
                Bound {
                    min: -1.0,
                    max: 1001.0,
                },
            ],
            1.0,
            10,
        );

        for node in nodes.into_iter() {
            tree.add_node(node).unwrap();
        }
    }));
}

#[bench]
fn bench_single_thread(bench: &mut Bencher) {
    // 266ms, rayon is almost no overhead! damn!
    // 464ms on Ryzen
    let mut rng = rand::thread_rng();
    bench.iter(black_box(|| {
        let mut nodes = vec![];
        for i in 0..1000 {
            for j in 0..1000 {
                nodes.push(Node::new_point(
                    [(rng.gen::<f64>()) * 1000., rng.gen::<f64>() * 1000.],
                    i * 1000 + j,
                ));
            }
        }

        GenericTree::<f64, 2, usize>::from_nodes(nodes, 1.0, 10);
    }));
}
