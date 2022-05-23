use crate::data::PointData;
use crate::force::ForceSimulate;
use num::Float;

pub struct CenterForce<F: Float, const N: usize> {
    pub target_position: [F; N],
    pub strength: F,
}

impl<F: Float, const N: usize> CenterForce<F, N> {
    pub fn new(target_position: [F; N], strength: F) -> CenterForce<F, N> {
        CenterForce {
            target_position,
            strength,
        }
    }

    pub fn set_strength(&mut self, strength: F) {
        self.strength = strength;
    }

    pub fn set_target_position(&mut self, target_position: [F; N]) {
        self.target_position = target_position;
    }
}

impl<F: Float, const N: usize> Default for CenterForce<F, N> {
    fn default() -> Self {
        CenterForce {
            target_position: [F::zero(); N],
            strength: F::one(),
        }
    }
}

impl<F: Float, const N: usize, D> ForceSimulate<F, N, D> for CenterForce<F, N> {
    fn init(&mut self, _: &[PointData<F, N, D>]) {}

    fn force(&self, force_point_data: &mut [PointData<F, N, D>], _: F) {
        let n = F::from(force_point_data.len() as f64).unwrap();
        let mut s = [F::zero(); N];
        for point_data in force_point_data.iter_mut() {
            for i in 0..N {
                s[i] = s[i] + point_data.coord[i];
            }
        }

        for i in 0..N {
            s[i] = (s[i] / n - self.target_position[i]) * self.strength;
        }

        for point_data in force_point_data.iter_mut() {
            for i in 0..N {
                point_data.coord[i] = point_data.coord[i] - s[i];
            }
        }
    }
}
