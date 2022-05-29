use crate::data::PointData;
use crate::force::ForceSimulate;
use num::Float;
use std::fmt::Display;

pub struct PositionForce<F: Float, const N: usize, D> {
    /// 目标点，如果对应维度是None则表示不进行对应维度力的模拟
    pub target_position_fn: fn(&PointData<F, N, D>, &[PointData<F, N, D>]) -> [Option<F>; N],
    pub strength_fn: fn(&PointData<F, N, D>, &[PointData<F, N, D>]) -> [Option<F>; N],
    target_position: Vec<[Option<F>; N]>,
    strengths: Vec<[Option<F>; N]>,
    force_point_data: Option<*const [PointData<F, N, D>]>,
}

impl<F: Float, const N: usize, D> PositionForce<F, N, D> {
    pub fn new(
        target_position_fn: fn(&PointData<F, N, D>, &[PointData<F, N, D>]) -> [Option<F>; N],
        strength_fn: fn(&PointData<F, N, D>, &[PointData<F, N, D>]) -> [Option<F>; N],
    ) -> PositionForce<F, N, D> {
        PositionForce {
            target_position_fn,
            strength_fn,
            target_position: Vec::new(),
            strengths: Vec::new(),
            force_point_data: None,
        }
    }

    fn _set_target_position(&mut self) {
        if let Some(force_point_data) = self.force_point_data {
            unsafe {
                for point_data in &*force_point_data {
                    self.target_position[point_data.index] =
                        (self.target_position_fn)(point_data, &*force_point_data)
                }
            }
        }
    }

    pub fn set_target_position_fn(
        &mut self,
        target_position_fn: fn(&PointData<F, N, D>, &[PointData<F, N, D>]) -> [Option<F>; N],
    ) {
        self.target_position_fn = target_position_fn;
        self._set_target_position();
    }

    pub fn set_strength_fn(
        &mut self,
        strength_fn: fn(&PointData<F, N, D>, &[PointData<F, N, D>]) -> [Option<F>; N],
    ) {
        self.strength_fn = strength_fn;
        self._set_strength();
    }

    fn _set_strength(&mut self) {
        if let Some(force_point_data) = self.force_point_data {
            unsafe {
                for point_data in &*force_point_data {
                    self.strengths[point_data.index] =
                        (self.strength_fn)(point_data, &*force_point_data)
                }
            }
        }
    }
}

impl<F: Float, const N: usize, D> Default for PositionForce<F, N, D> {
    fn default() -> Self {
        PositionForce {
            target_position_fn: |_, _| [Some(F::zero()); N],
            strength_fn: |_, _| [Some(F::from(0.1f64).unwrap()); N],
            target_position: Vec::new(),
            strengths: Vec::new(),
            force_point_data: None,
        }
    }
}

impl<F: Float + Send + Sync, const N: usize, D: Default + Display + Clone + Send + Sync>
    ForceSimulate<F, N, D> for PositionForce<F, N, D>
{
    fn init(&mut self, force_point_data: &[PointData<F, N, D>]) {
        self.force_point_data = Some(force_point_data as *const [PointData<F, N, D>]);
        self.target_position = vec![[None; N]; force_point_data.len()];
        self.strengths = vec![[None; N]; force_point_data.len()];
        self._set_target_position();
        self._set_strength();
    }

    fn force(&self, force_point_data: &mut [PointData<F, N, D>], alpha: F) {
        force_point_data.iter_mut().for_each(|point_data| {
            for i in 0..N {
                if let (Some(target_position_i), Some(strengths_i)) = (
                    self.target_position[point_data.index][i],
                    self.strengths[point_data.index][i],
                ) {
                    let _delta_i = (target_position_i - point_data.coord[i]) * strengths_i * alpha;
                    point_data.velocity[i] = point_data.velocity[i] + _delta_i;
                }
            }
        })
    }
}
