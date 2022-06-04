use crate::utils::{draw_node, RandomData};
use plotters::coord::types::RangedCoordf32;
use plotters::prelude::*;
use simulation::force::{CollideForce, RadialForce};
use simulation::Simulation;
use std::time::Instant;

mod utils;

fn build_radial(data_len: usize) -> RadialForce<f64, 2> {
    let sl = data_len / 2;
    let mut r = vec![100f64; sl];
    r.append(&mut vec![50f64; data_len - sl]);
    RadialForce::new(r, [0f64; 2], vec![0.3f64; data_len])
}

fn build_simulation(data: Vec<RandomData>) -> Simulation<f64, 2, RandomData> {
    let data_len = data.len();
    let mut simulation: Simulation<f64, 2, RandomData> = Simulation::from_data(data);

    // collide force
    let collide_force: CollideForce<f64, 2, 4, RandomData> =
        CollideForce::new(vec![2_f64; data_len], 0.6_f64, 1);
    simulation.add_force(String::from("collide"), Box::new(collide_force));

    let radial_force = build_radial(data_len);
    simulation.add_force(String::from("radial"), Box::new(radial_force));
    simulation
}

fn main() {
    let area = BitMapBackend::gif("radial-force.gif", (640, 640), 300)
        .unwrap()
        .into_drawing_area();

    let area = area.apply_coord_spec(Cartesian2d::<RangedCoordf32, RangedCoordf32>::new(
        -144f32..144f32,
        -144f32..144f32,
        (0..640, 0..640),
    ));
    let node_num = 100;
    let data = vec![RandomData::default(); node_num];
    let mut sim = build_simulation(data);
    for i in 0..30 {
        println!("draw {}/30", i + 1);
        area.fill(&RGBColor(240, 144, 144)).unwrap();
        draw_node(&mut sim, &area);
        area.present().unwrap();
        let start = Instant::now();
        sim.tick();
        println!("tick {} ms", start.elapsed().as_millis());
    }
}
