#![feature(ptr_internals)]
use plotters::coord::types::RangedCoordf32;
use plotters::coord::Shift;
use plotters::element::*;
use plotters::prelude::*;
use rand::prelude::*;
use simulation::force::{CenterForce, LinkForce, NBodyForce, PositionForce};
use simulation::Simulation;
use std::fmt::{Display, Formatter};
use std::time::Instant;

#[derive(Clone)]
struct RandomData {
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

fn build_random_links(data: &[RandomData]) -> Vec<(usize, usize)> {
    let mut links = Vec::new();
    for (source_index, _) in data.iter().enumerate() {
        if thread_rng().gen_range(0..10) < 1 {
            for (target_index, _) in data.iter().enumerate() {
                if source_index != target_index && thread_rng().gen_range(0..10) < 1 {
                    links.push((source_index, target_index))
                }
            }
        }
    }
    links
}

fn build_simulation(
    data: Vec<RandomData>,
    links: Vec<(usize, usize)>,
) -> Simulation<f64, 2, RandomData> {
    let mut simulation: Simulation<f64, 2, RandomData> = Simulation::from_data(data);
    let mut nbody_force: NBodyForce<f64, 2, 4, RandomData> = NBodyForce::default();
    nbody_force.set_strength_fn(|_, _| -50_f64);
    simulation.add_force(String::from("n-body"), Box::new(nbody_force));
    let mut position_force = PositionForce::default();
    position_force.set_strength_fn(|_, _| [Some(1f64); 2]);
    simulation.add_force(String::from("position"), Box::new(position_force));
    simulation.add_force(String::from("center"), Box::new(CenterForce::default()));
    let mut link_force = LinkForce::default();
    link_force.set_links(links);
    link_force.set_distance_fn(|_, _| 0_f64);
    link_force.set_strength_fn(|_, _| 1_f64);
    simulation.add_force(String::from("link"), Box::new(link_force));
    simulation
}

fn draw_node(
    sim: &mut Simulation<f64, 2, RandomData>,
    area: &DrawingArea<BitMapBackend, Cartesian2d<RangedCoordf32, RangedCoordf32>>,
) {
    let dot_and_label = |x: f32, y: f32| {
        return EmptyElement::at((x, y))
            + Circle::new((0, 0), 3, ShapeStyle::from(&BLACK).filled())
            + Text::new(
                format!("({:.2},{:.2})", x, y),
                (10, 0),
                ("sans-serif", 15.0).into_font(),
            );
    };
    sim.force_point_data.iter().for_each(|data| {
        area.draw(&dot_and_label(data.coord[0] as f32, data.coord[1] as f32))
            .unwrap()
    });
}

fn draw_line(
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

fn main() {
    let area = BitMapBackend::gif("nbody-force.gif", (640, 640), 300)
        .unwrap()
        .into_drawing_area();
    let mut chart = ChartBuilder::on(&area)
        .build_cartesian_2d(-128f32..128f32, -128f32..128f32)
        .unwrap();

    let area = area.apply_coord_spec(Cartesian2d::<RangedCoordf32, RangedCoordf32>::new(
        -128f32..128f32,
        -128f32..128f32,
        (0..640, 0..640),
    ));
    let node_num = 50;
    let data = vec![RandomData::default(); node_num];
    let links = build_random_links(&data);
    let mut sim = build_simulation(data, links.clone());
    for i in 0..30 {
        println!("draw {}/30", i + 1);
        area.fill(&RGBColor(240, 200, 200)).unwrap();
        draw_node(&mut sim, &area);
        draw_line(&mut sim, links.clone(), &mut chart);
        area.present().unwrap();
        let start = Instant::now();
        sim.tick();
        println!("tick {} ms", start.elapsed().as_millis());
    }
}
