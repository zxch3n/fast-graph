use std::fmt::{Debug, Display};

pub trait TreeData {
    type PointData: Send + Sync + Clone + Default + Display;
    type RegionData: Send + Sync + Clone + Default + Display;

    fn merge_point_data(&self, p: &[Self::PointData]) -> Self::RegionData {
        Self::RegionData::default()
    }

    fn merge_region_data(&self, p: &[Self::RegionData]) -> Self::RegionData {
        Self::RegionData::default()
    }
}
