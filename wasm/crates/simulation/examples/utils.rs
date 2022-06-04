use plotters::coord::types::RangedCoordf32;
use plotters::coord::Shift;
use plotters::prelude::*;
use rand::{thread_rng, Rng};
use simulation::Simulation;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct RandomData {
    data: (),
}

impl Default for RandomData {
    fn default() -> Self {
        RandomData { data: () }
    }
}

impl Display for RandomData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

pub fn build_random_links(data: &[RandomData]) -> Vec<(usize, usize)> {
    let mut links = Vec::new();
    for source_index in 0..data.len() {
        if thread_rng().gen_range(0..15) < 1 {
            for target_index in source_index..data.len() {
                if source_index < target_index && thread_rng().gen_range(0..15) < 1 {
                    links.push((source_index, target_index))
                }
            }
        }
    }
    links
}

pub fn draw_node(
    sim: &mut Simulation<f64, 2, RandomData>,
    area: &DrawingArea<BitMapBackend, Cartesian2d<RangedCoordf32, RangedCoordf32>>,
) {
    let dot_and_label = |x: f32, y: f32| {
        return EmptyElement::at((x, y))
            + Circle::new((0, 0), 3, ShapeStyle::from(&BLACK).filled());
        // + Text::new(
        //     format!("({:.2},{:.2})", x, y),
        //     (10, 0),
        //     ("sans-serif", 15.0).into_font(),
        // );
    };
    sim.force_point_data.iter().for_each(|data| {
        area.draw(&dot_and_label(data.coord[0] as f32, data.coord[1] as f32))
            .unwrap()
    });
}

pub fn draw_line(
    sim: &mut Simulation<f64, 2, RandomData>,
    links: Vec<(usize, usize)>,
    chart: &mut ChartContext<BitMapBackend, Cartesian2d<RangedCoordf32, RangedCoordf32>>,
) {
    for (s, t) in links {
        // 负号因为chart中y是反向
        let from = (
            sim.force_point_data[s].coord[0] as f32,
            -sim.force_point_data[s].coord[1] as f32,
        );
        let to = (
            sim.force_point_data[t].coord[0] as f32,
            -sim.force_point_data[t].coord[1] as f32,
        );
        chart
            .draw_series(LineSeries::new([from, to].into_iter(), &BLACK))
            .unwrap();
    }
}

fn main() {}
