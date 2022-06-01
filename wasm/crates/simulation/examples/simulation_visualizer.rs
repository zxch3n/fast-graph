#![feature(ptr_internals)]
use plotters::coord::types::RangedCoordf32;
use plotters::element::*;
use plotters::prelude::*;
use rand::prelude::*;
use simulation::force::{CenterForce, CollideForce, LinkForce, NBodyForce, PositionForce};
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

fn build_simulation(
    data: Vec<RandomData>,
    links: Vec<(usize, usize)>,
) -> Simulation<f64, 2, RandomData> {
    let data_len = data.len();
    let link_len = links.len();
    let mut simulation: Simulation<f64, 2, RandomData> = Simulation::from_data(data);

    // nbody force
    let mut nbody_force: NBodyForce<f64, 2, 4, RandomData> = NBodyForce::new(0f64, 1e7f64, 0.9f64);
    nbody_force.strengths = vec![-30_f64; data_len];
    simulation.add_force(String::from("n-body"), Box::new(nbody_force));

    // position force
    let position_force = PositionForce::new(
        vec![[Some(0_f64); 2]; data_len],
        vec![[Some(1_f64); 2]; data_len],
    );
    simulation.add_force(String::from("position"), Box::new(position_force));

    // center force
    simulation.add_force(String::from("center"), Box::new(CenterForce::default()));

    // link force
    let link_force = LinkForce::new(links, vec![1_f64; link_len], vec![0_f64; link_len], 1);
    simulation.add_force(String::from("link"), Box::new(link_force));

    // collide force
    let collide_force: CollideForce<f64, 2, 4, RandomData> =
        CollideForce::new(vec![2_f64; data_len], 0.6_f64, 1);
    simulation.add_force(String::from("collide"), Box::new(collide_force));
    simulation
}

fn draw_node(
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
        .build_cartesian_2d(-144f32..144f32, -144f32..144f32)
        .unwrap();

    let area = area.apply_coord_spec(Cartesian2d::<RangedCoordf32, RangedCoordf32>::new(
        -144f32..144f32,
        -144f32..144f32,
        (0..640, 0..640),
    ));
    let node_num = 100;
    let data = vec![RandomData::default(); node_num];
    let links = build_random_links(&data);
    let mut sim = build_simulation(data, links.clone());
    for i in 0..30 {
        println!("draw {}/30", i + 1);
        area.fill(&RGBColor(240, 144, 144)).unwrap();
        draw_node(&mut sim, &area);
        draw_line(&mut sim, links.clone(), &mut chart);
        area.present().unwrap();
        let start = Instant::now();
        sim.tick();
        println!("tick {} ms", start.elapsed().as_millis());
    }
}
