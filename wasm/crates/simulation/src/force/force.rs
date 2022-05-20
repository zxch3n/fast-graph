use crate::data::PointData;
use num::Float;

pub trait ForceSimulate<F: Float, const N: usize, D> {
    fn init(&mut self, force_point_data: &[PointData<F, N, D>]);
    fn force(&self, force_point_data: &mut [PointData<F, N, D>], alpha: F);
}
