//! A basic test to compare the ray fan with a verified version
use std::f64::consts::PI;

use super::sphere_ray_tracer::SphereRayTracer;

#[test]
fn sphere_geodesics_test() {
    let mut sphere = SphereRayTracer::new(100., 10., 100, PI/100., 10);
    let result = sphere.solve_ray_fan(25.);
    let stall = true;
}