use generic_tree::TreeData;
use num::Float;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

pub struct ForceData<F, const N: usize, D> {
    _float_marker: PhantomData<F>,
    _data_marker: PhantomData<D>,
}

#[derive(Clone)]
pub struct PointForceData<F: Float, const N: usize, D> {
    pub data: D,
    pub index: usize,
    pub coord: [F; N],
    pub velocity: [F; N],
    pub strength: F,
    pub fixed_position: Option<[F; N]>,
}

#[derive(Clone)]
pub struct RegionForceData<F: Float, const N: usize> {
    /// weighted coord
    pub coord: [F; N],
    pub strength: F,
}

impl<F: Float, const N: usize, D: Default> Default for PointForceData<F, N, D> {
    fn default() -> Self {
        Self {
            data: D::default(),
            index: 0,
            coord: [F::zero(); N],
            velocity: [F::zero(); N],
            strength: F::zero(),
            fixed_position: None,
        }
    }
}

impl<F: Float, const N: usize, D: Display> Display for PointForceData<F, N, D> {
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

impl<F: Float, const N: usize> Display for RegionForceData<F, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RegionForceData(coord: {:?}, strength: {:?})",
            self.coord
                .iter()
                .map(|point| point.to_f64().unwrap())
                .collect::<Vec<_>>(),
            self.strength.to_f64().unwrap()
        )
    }
}

unsafe impl<F: Float, const N: usize, D> Send for PointForceData<F, N, D> {}
unsafe impl<F: Float, const N: usize, D> Sync for PointForceData<F, N, D> {}

impl<F: Float, const N: usize> Default for RegionForceData<F, N> {
    fn default() -> Self {
        Self {
            coord: [F::zero(); N],
            strength: F::zero(),
        }
    }
}
unsafe impl<F: Float, const N: usize> Send for RegionForceData<F, N> {}
unsafe impl<F: Float, const N: usize> Sync for RegionForceData<F, N> {}

impl<F: Float, const N: usize, D: Display + Clone + Default> TreeData for ForceData<F, N, D> {
    type PointData = PointForceData<F, N, D>;
    type RegionData = RegionForceData<F, N>;

    fn merge_point_data(&self, p: &[Self::PointData]) -> Self::RegionData {
        todo!()
    }

    fn merge_region_data(&self, p: &[Self::RegionData]) -> Self::RegionData {
        todo!()
    }
}

impl<F: Float, const N: usize, D> PointForceData<F, N, D> {
    pub fn from_data(data: D, coord: [F; N], index: usize) -> PointForceData<F, N, D> {
        Self {
            data,
            coord,
            index,
            velocity: [F::zero(); N],
            strength: F::zero(),
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
