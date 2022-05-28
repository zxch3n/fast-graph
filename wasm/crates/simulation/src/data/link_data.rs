use crate::data::PointData;
use num::Float;

pub struct LinkData<F: Float, const N: usize, D> {
    pub index: usize,
    source: *const PointData<F, N, D>,
    target: *const PointData<F, N, D>,
}

impl<F: Float, const N: usize, D> LinkData<F, N, D> {
    pub fn new(
        index: usize,
        source: &mut PointData<F, N, D>,
        target: &mut PointData<F, N, D>,
    ) -> LinkData<F, N, D> {
        LinkData {
            index,
            source,
            target,
        }
    }

    // TODO 更好的from方案
    pub fn from_pairs(
        pairs: &[(usize, usize)],
        point_data: &[PointData<F, N, D>],
    ) -> Vec<LinkData<F, N, D>> {
        let mut link_data = Vec::with_capacity(pairs.len());
        // TODO 先构建mapping？
        for (index, (s, t)) in pairs.iter().enumerate() {
            // TODO unwrap可能出错
            let source = point_data.iter().find(|&d| d.index == *s).unwrap();
            let target = point_data.iter().find(|&d| d.index == *t).unwrap();
            link_data.push(LinkData {
                index,
                source,
                target,
            })
        }
        link_data
    }

    pub fn source(&self) -> &PointData<F, N, D> {
        unsafe { &(*self.source) }
    }

    pub fn target(&self) -> &PointData<F, N, D> {
        unsafe { &(*self.target) }
    }
}
