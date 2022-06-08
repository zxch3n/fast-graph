use crate::data::PointData;
use crate::force::ForceSimulate;
use num::Float;

pub struct RadialForce<F: Float, const N: usize> {
    pub radiuses: Vec<F>,
    pub center: [F; N],
    pub strengths: Vec<F>,
}

impl<F: Float, const N: usize> RadialForce<F, N> {
    pub fn new(radiuses: Vec<F>, center: [F; N], strengths: Vec<F>) -> RadialForce<F, N> {
        RadialForce {
            radiuses,
            center,
            strengths,
        }
    }
}

impl<F: Float, const N: usize, D> ForceSimulate<F, N, D> for RadialForce<F, N> {
    fn init(&mut self, _: &[PointData<F, N, D>]) {
        ()
    }

    fn force(&self, force_point_data: &mut [PointData<F, N, D>], alpha: F) {
        for (index, force_point_datum) in force_point_data.iter_mut().enumerate() {
            let mut delta = [F::zero(); N];
            for i in 0..N {
                delta[i] = force_point_datum.coord[i] - self.center[i]
            }
            let r = delta.iter().fold(F::zero(), |s, &x| s + x * x).sqrt() + F::epsilon();
            let k = (self.radiuses[index] - r) * self.strengths[index] * alpha / r;
            for i in 0..N {
                force_point_datum.velocity[i] = force_point_datum.velocity[i] + delta[i] * k;
            }
        }
    }
}
