use num::Float;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct PointData<F: Float, const N: usize, D> {
    pub data: D,
    pub index: usize,
    pub coord: [F; N],
    pub velocity: [F; N],
    pub strength: F,
    pub radius: F,
    pub fixed_position: Option<[F; N]>,
}

impl<F: Float, const N: usize, D> PointData<F, N, D> {
    pub fn from_data(data: D, coord: [F; N], index: usize) -> PointData<F, N, D> {
        Self {
            data,
            coord,
            index,
            velocity: [F::zero(); N],
            strength: F::zero(),
            radius: F::zero(),
            fixed_position: None,
        }
    }

    pub fn coord(&self) -> &[F; N] {
        &self.coord
    }

    pub fn coord_mut(&mut self) -> &mut [F; N] {
        &mut self.coord
    }
}

impl<F: Float, const N: usize, D: Default> Default for PointData<F, N, D> {
    fn default() -> Self {
        Self {
            data: D::default(),
            index: 0,
            coord: [F::zero(); N],
            velocity: [F::zero(); N],
            strength: F::zero(),
            radius: F::zero(),
            fixed_position: None,
        }
    }
}

impl<F: Float, const N: usize, D: Display> Display for PointData<F, N, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PointForceData(data: {}, coord: {:?}, strength: {:?}, velocity: {:?}), fixed_position: {:?}",
            self.data,
            self.coord
                .iter()
                .map(|point| point.to_f64().unwrap())
                .collect::<Vec<_>>(),
            self.strength.to_f64().unwrap(),
            self.velocity
                .iter()
                .map(|point| point.to_f64().unwrap())
                .collect::<Vec<_>>(),
            match self.fixed_position {
                None => None,
                Some(position) => Some(position.iter()
                    .map(|point| point.to_f64().unwrap())
                    .collect::<Vec<_>>())
            }
        )
    }
}
