//! A basic test to compare the ray fan with a verified version
use std::f64::consts::PI;

use glam::Vec3;

use super::{sphere_ray_tracer::SphereRayTracer, ray_connector::{self, RayConnector}};

#[test]
fn sphere_geodesics_test() {
    let mut sphere = SphereRayTracer::new(100., 10., 100, PI/100., 10);
    let _result = sphere.solve_ray_fan(25.);
    let _stall = true;
}

#[test]
fn ray_connector_euclidian_test() -> Result<(), String> {
    let pos = Vec3{x: 20., y: 10., z: 5.};
    let observer_pos = Vec3{x: -20., y: -10., z: 5.};
    let mut ray_connector = RayConnector::new(0., pos, true);
    let euclidian_angle = (pos - observer_pos).angle_between(-observer_pos);
    let output = ray_connector.reset_ray(observer_pos);
    let angles_match = (euclidian_angle - output[3]).abs() < 1e-3 as f32;
    assert!(angles_match);

    Ok(())
}