use std::fmt::{Debug, Display};

pub trait TreeData {
    /// Drop trait implementation in [`TreeData::PointData`] won't be invoked.
    /// So you cannot use struct like Box/Vec/Rc here.
    /// Instead you can use [bumpalo::collections::Vec] or [bumpalo::boxed::Box]
    type PointData: Send + Sync + Clone + Display;
    /// Drop trait implementation in [`TreeData::RegionData`] won't be invoked.
    /// So you cannot use struct like Box/Vec/Rc here.
    /// Instead you can use [bumpalo::collections::Vec] or [bumpalo::boxed::Box]
    type RegionData: Send + Sync + Clone + Default + Display;

    fn merge_point_data(&self, p: &[Self::PointData]) -> Self::RegionData {
        Self::RegionData::default()
    }

    fn merge_region_data(&self, p: &[Self::RegionData]) -> Self::RegionData {
        Self::RegionData::default()
    }
}
