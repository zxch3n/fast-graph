use crate::data::{LinkData, PointData};
use crate::force::utils::{about_zero, jiggle};
use crate::force::ForceSimulate;
use num::Float;

pub struct LinkForce<F: Float, const N: usize, D> {
    pub links: Vec<(usize, usize)>,
    pub strengths: Vec<F>,
    pub distances: Vec<F>,
    pub iterations: usize,
    count: Vec<usize>,
    bias: Vec<F>,
    links_data: Vec<LinkData<F, N, D>>,
}

impl<F: Float, const N: usize, D> LinkForce<F, N, D> {
    pub fn new(
        links: Vec<(usize, usize)>,
        strengths: Vec<F>,
        distances: Vec<F>,
        iterations: usize,
    ) -> LinkForce<F, N, D> {
        LinkForce {
            links,
            iterations,
            strengths,
            distances,
            links_data: Vec::new(),
            count: Vec::new(),
            bias: Vec::new(),
        }
    }
}

unsafe fn split_borrow_two_diff_index<F: Float, const N: usize, D>(
    data: &mut [PointData<F, N, D>],
    source_index: usize,
    target_index: usize,
) -> (&mut PointData<F, N, D>, &mut PointData<F, N, D>) {
    assert_ne!(source_index, target_index);
    let ptr = data.as_mut_ptr();
    let source = ptr.add(source_index).as_mut().unwrap();
    let target = ptr.add(target_index).as_mut().unwrap();
    (source, target)
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
    }

    fn force(&self, force_point_data: &mut [PointData<F, N, D>], alpha: F) {
        let mut rnd = rand::thread_rng();
        for _ in 0..self.iterations {
            for link in self.links_data.iter() {
                let (mut source, mut target) = unsafe {
                    split_borrow_two_diff_index(
                        force_point_data,
                        link.source().index,
                        link.target().index,
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
