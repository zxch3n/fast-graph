use crate::data::PointData;
use crate::force::ForceSimulate;
use num::Float;
use std::fmt::Display;

pub struct PositionForce<F: Float, const N: usize, D> {
    /// 目标点，如果对应维度是None则表示不进行对应维度力的模拟
    pub target_position: Vec<[Option<F>; N]>,
    pub strengths: Vec<[Option<F>; N]>,
    force_point_data: Option<*const [PointData<F, N, D>]>,
}

impl<F: Float, const N: usize, D> PositionForce<F, N, D> {
    pub fn new(
        target_position: Vec<[Option<F>; N]>,
        strengths: Vec<[Option<F>; N]>,
    ) -> PositionForce<F, N, D> {
        PositionForce {
            target_position,
            strengths,
            force_point_data: None,
        }
    }
}

impl<F: Float + Send + Sync, const N: usize, D: Default + Display + Clone + Send + Sync>
    ForceSimulate<F, N, D> for PositionForce<F, N, D>
{
    fn init(&mut self, force_point_data: &[PointData<F, N, D>]) {
        self.force_point_data = Some(force_point_data as *const [PointData<F, N, D>]);
    }

    fn force(&self, force_point_data: &mut [PointData<F, N, D>], alpha: F) {
        for (index, force_point_datum) in force_point_data.iter_mut().enumerate() {
            for i in 0..N {
                if let (Some(target_position_i), Some(strengths_i)) =
                    (self.target_position[index][i], self.strengths[index][i])
                {
                    let _delta_i =
                        (target_position_i - force_point_datum.coord[i]) * strengths_i * alpha;
                    force_point_datum.velocity[i] = force_point_datum.velocity[i] + _delta_i;
                }
            }
        }
    }
}
