use crate::data::point_data::PointData;
use generic_tree::TreeData;
use num::Float;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::Unique;

pub struct ForceData<F, const N: usize, D> {
    _marker: PhantomData<(F, D)>,
}

impl<F: Float + Send + Sync, const N: usize, D: Display + Clone + Default + Send + Sync> TreeData
    for ForceData<F, N, D>
{
    type PointData = PointForceData<F, N, D>;
    type RegionData = RegionForceData<F, N>;

    fn merge_point_data(&self, p: &[Self::PointData]) -> Self::RegionData {
        todo!()
    }

    fn merge_region_data(&self, p: &[Self::RegionData]) -> Self::RegionData {
        todo!()
    }
}

#[derive(Clone)]
pub struct PointForceData<F: Float, const N: usize, D> {
    ptr: Unique<PointData<F, N, D>>,
}

impl<F: Float, const N: usize, D> PointForceData<F, N, D> {
    pub fn from_point_data(point_data: &mut PointData<F, N, D>) -> PointForceData<F, N, D> {
        PointForceData {
            ptr: Unique::new(point_data).unwrap(),
        }
    }
}

impl<F: Float, const N: usize, D: Default> Display for PointForceData<F, N, D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &(self.ptr.as_ptr()))
    }
}

impl<F: Float, const N: usize, D: Default> Deref for PointForceData<F, N, D> {
    type Target = PointData<F, N, D>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<F: Float, const N: usize, D: Default> DerefMut for PointForceData<F, N, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

#[derive(Clone)]
pub struct RegionForceData<F: Float, const N: usize> {
    /// weighted coord
    pub coord: Option<[F; N]>,
    pub strength: Option<F>,
    pub radius: Option<F>,
}

impl<F: Float + Send + Sync, const N: usize> Display for RegionForceData<F, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RegionForceData(coord: {:?}, strength: {:?})",
            self.coord
                .unwrap_or([F::nan(); N])
                .iter()
                .map(|point| point.to_f64().unwrap())
                .collect::<Vec<_>>(),
            self.strength.unwrap_or(F::nan()).to_f64().unwrap()
        )
    }
}

impl<F: Float + Send + Sync, const N: usize> Default for RegionForceData<F, N> {
    fn default() -> Self {
        Self {
            coord: None,
            strength: None,
            radius: None,
        }
    }
}
