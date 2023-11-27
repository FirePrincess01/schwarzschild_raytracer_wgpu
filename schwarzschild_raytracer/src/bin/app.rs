
//! The main function of the program
fn main() {
    pollster::block_on(schwarzschild_raytracer::run());
}