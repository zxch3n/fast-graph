use crate::data::{LinkData, PointData};
use crate::force::utils::{about_zero, jiggle};
use crate::force::ForceSimulate;
use num::Float;
use std::cmp::min;
use std::ops::{Index, IndexMut};
use std::slice::from_raw_parts_mut;

pub struct LinkForce<F: Float, const N: usize, D> {
    pub links: Vec<(usize, usize)>,
    links_data: Vec<LinkData<F, N, D>>,
    // FIXME &LinkForce<F, N, D>参数是否有更好的初始化办法    ->  使用Box<dyn Fn>？
    pub strength_fn: fn(&LinkData<F, N, D>, &LinkForce<F, N, D>) -> F,
    strengths: Vec<F>,
    pub distance_fn: fn(&LinkData<F, N, D>, &[LinkData<F, N, D>]) -> F,
    distances: Vec<F>,
    count: Vec<usize>,
    bias: Vec<F>,
    iterations: usize,
}

impl<F: Float, const N: usize, D> LinkForce<F, N, D> {
    pub fn new(
        links: Vec<(usize, usize)>,
        strength_fn: fn(&LinkData<F, N, D>, &LinkForce<F, N, D>) -> F,
        distance_fn: fn(&LinkData<F, N, D>, &[LinkData<F, N, D>]) -> F,
        iterations: usize,
    ) -> LinkForce<F, N, D> {
        LinkForce {
            links,
            strength_fn,
            distance_fn,
            iterations,
            links_data: Vec::new(),
            strengths: Vec::new(),
            distances: Vec::new(),
            count: Vec::new(),
            bias: Vec::new(),
        }
    }

    fn init_strengths(&mut self) {
        for link in self.links_data.iter() {
            self.strengths[link.index] = (self.strength_fn)(link, self)
        }
    }

    fn init_distances(&mut self) {
        for link in self.links_data.iter() {
            self.distances[link.index] = (self.distance_fn)(link, self.links_data.as_slice())
        }
    }

    pub fn set_links(&mut self, links: Vec<(usize, usize)>) {
        self.links = links;
    }

    pub fn count(&self) -> &[usize] {
        &self.count
    }
}

fn default_strength_fn<F: Float, const N: usize, D>(
    link: &LinkData<F, N, D>,
    force: &LinkForce<F, N, D>,
) -> F {
    F::one()
        / F::from(min(
            force.count()[link.source().index],
            force.count()[link.target().index],
        ))
        .unwrap()
}

unsafe fn split_borrow_two_diff_index<F: Float, const N: usize, D>(
    data: &mut [PointData<F, N, D>],
    source_index: usize,
    target_index: usize,
) -> (&mut PointData<F, N, D>, &mut PointData<F, N, D>) {
    assert_ne!(source_index, target_index);
    let ptr = data.as_mut_ptr();
    let source = from_raw_parts_mut(ptr.add(source_index), 1).index_mut(0);
    let target = from_raw_parts_mut(ptr.add(target_index), 1).index_mut(0);
    (source, target)
}

impl<F: Float, const N: usize, D> Default for LinkForce<F, N, D> {
    fn default() -> Self {
        LinkForce {
            links: Vec::new(),
            links_data: Vec::new(),
            strength_fn: default_strength_fn,
            distance_fn: |_, _| F::from(30_f64).unwrap(),
            strengths: Vec::new(),
            distances: Vec::new(),
            count: Vec::new(),
            bias: Vec::new(),
            iterations: 1,
        }
    }
}

impl<F: Float, const N: usize, D> ForceSimulate<F, N, D> for LinkForce<F, N, D> {
    fn init(&mut self, force_point_data: &[PointData<F, N, D>]) {
        self.count = vec![0; force_point_data.len()];
        self.bias = vec![F::zero(); self.links.len()];

        self.links_data = LinkData::from_pairs(&self.links, force_point_data);

        for link in self.links_data.iter() {
            self.count[link.source().index] = self.count[link.source().index] + 1;
            self.count[link.target().index] = self.count[link.target().index] + 1;
        }
        for link in self.links_data.iter() {
            self.bias[link.index] = F::from(self.count[link.source().index]).unwrap()
                / F::from(self.count[link.source().index] + self.count[link.target().index])
                    .unwrap();
        }
        self.strengths = vec![F::zero(); self.links_data.len()];
        self.distances = vec![F::zero(); self.links_data.len()];
        self.init_strengths();
        self.init_distances();
    }

    fn force(&self, force_point_data: &mut [PointData<F, N, D>], alpha: F) {
        let mut rnd = rand::thread_rng();
        for _ in 0..self.iterations {
            for link in self.links_data.iter() {
                let source_index = link.source().index;
                let target_index = link.target().index;
                // TODO find会不会效率问题？
                let (mut real_source_index, mut real_target_index) = (0, 0);
                for (idx, force_point_datum) in force_point_data.iter().enumerate() {
                    if force_point_datum.index == source_index {
                        real_source_index = idx
                    }
                    if force_point_datum.index == target_index {
                        real_target_index = idx
                    }
                }
                let (mut source, mut target) = unsafe {
                    split_borrow_two_diff_index(
                        force_point_data,
                        real_source_index,
                        real_target_index,
                    )
                };

                let mut p = [F::zero(); N];
                for i in 0..N {
                    p[i] =
                        target.coord[i] + target.velocity[i] - source.coord[i] - source.velocity[i];
                    if about_zero(p[i]) {
                        p[i] = jiggle(&mut rnd)
                    }
                }
                let mut l = p.iter().fold(F::zero(), |s, &x| s + x * x).sqrt();
                l = (l - self.distances[link.index]) / l * alpha * self.strengths[link.index];
                for i in 0..N {
                    p[i] = p[i] * l
                }
                let mut bias = self.bias[link.index];
                for i in 0..N {
                    target.velocity[i] = target.velocity[i] - p[i] * bias
                }
                bias = F::one() - bias;
                for i in 0..N {
                    source.velocity[i] = source.velocity[i] + p[i] * bias
                }
            }
        }
    }
}
