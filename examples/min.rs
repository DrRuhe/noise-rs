extern crate noise;

use noise::{utils::*, Cylinders, Min, Perlin};

mod utils;

fn main() {
    let cyl = Cylinders::new();
    let perlin = Perlin::default();
    let min = Min::new(cyl, perlin);

    utils::write_example_to_file(&PlaneMapBuilder::<_, 3>::new(min).build(), "min.png");
}
