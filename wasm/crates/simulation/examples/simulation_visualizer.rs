#![feature(ptr_internals)]
use plotters::coord::types::RangedCoordf32;
use plotters::prelude::*;
use simulation::force::{CenterForce, CollideForce, LinkForce, NBodyForce, PositionForce};
use simulation::Simulation;
use std::time::Instant;

mod utils;
use utils::*;

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
