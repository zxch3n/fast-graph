#![feature(ptr_internals)]
use plotters::coord::types::RangedCoordf32;
use plotters::prelude::*;
use rand::prelude::*;
use simulation::force::{NBodyForce, PositionForce};
use simulation::Simulation;
use std::fmt::{Display, Formatter};
use std::time::Instant;

#[derive(Clone)]
struct RandomData {
    data: Vec<i32>,
}

impl Default for RandomData {
    fn default() -> Self {
        let num = thread_rng().gen_range(3..20);
        RandomData {
            data: Vec::from_iter(0..num),
        }
    }
}

impl Display for RandomData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.data)
    }
}

fn build_simulation<'s>(node_num: usize) -> Simulation<f64, 2, RandomData> {
    let mut data = Vec::with_capacity(node_num);
    for _ in 0..node_num {
        data.push(RandomData::default())
    }
    let mut simulation: Simulation<f64, 2, RandomData> = Simulation::from_data(data);
    simulation.add_force(
        String::from("n-body"),
        Box::new(NBodyForce::<f64, 2, 4, RandomData>::default()),
    );
    let mut position_force = PositionForce::default();
    position_force.set_strength_fn(|_, _| [Some(1f64); 2]);
    simulation.add_force(String::from("position"), Box::new(position_force));
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
    let start = Instant::now();
    sim.tick();
    println!("tick {} ms", start.elapsed().as_millis());
}

fn main() {
    let area = BitMapBackend::gif("nbody-force.gif", (640, 640), 300)
        .unwrap()
        .into_drawing_area();

    let area = area.apply_coord_spec(Cartesian2d::<RangedCoordf32, RangedCoordf32>::new(
        -128f32..128f32,
        -128f32..128f32,
        (0..640, 0..640),
    ));
    let mut sim = build_simulation(100);
    for i in 0..=30 {
        println!("draw {}/30", i);
        area.fill(&RGBColor(240, 200, 200)).unwrap();
        draw_node(&mut sim, &area);
        area.present().unwrap()
    }
}
