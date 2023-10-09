

fn main() {
    pollster::block_on(schwarzschild_raytracer::run());
}