//! A basic test to compare the ray fan with a verified version
use std::f64::consts::PI;

use glam::Vec3;

use super::{sphere_ray_tracer::SphereRayTracer, ray_connector::RayConnector};

#[test]
fn sphere_geodesics_test() {
    let mut sphere = SphereRayTracer::new(100., 10., 100, PI/100., 10);
    let _result = sphere.solve_ray_fan(25.);
    let _stall = true;
}

#[test]
fn ray_connector_euclidian_test() -> Result<(), String> {
    const NR_TESTS: usize = 100;
    let mut counter: usize = 0;
    for i in 0..NR_TESTS {
        let pos = Vec3{x: 20., y: 0., z: 0.1};
        let angle = i as f32 / NR_TESTS as f32 * std::f32::consts::PI;
        let observer_pos = Vec3{x: 19. * angle.cos(), y: 19. * angle.sin(), z: 0.};
        let mut ray_connector = RayConnector::new(0., pos, true);
        let euclidian_angle = (pos - observer_pos).angle_between(-observer_pos);
        let output = ray_connector.reset_ray(observer_pos);
        let error = (euclidian_angle - output[3]).abs();
        let angles_match = error < 5e-4 as f32; //error needs to be smaller than one pixel on a 4k display
        if angles_match {
            counter += 1;
        } 
        else {
            println!("Failed with error {error} at angle {angle}");
        }
    }
    assert_eq!(counter, NR_TESTS);
    // //println!("{}",&ray_connector.print_ray_for_matlab());

    Ok(())
}

// Checks if the ray connector can keep up with a moving observer with 1 iteration updates
#[test]
fn ray_connector_euclidian_tracing_test()  -> Result<(), String> {
    const NR_TESTS: usize = 60; // Fly around the black hole in one second (60 frames)
    let mut counter: usize = 0;
    let pos = Vec3{x: 20., y: 0., z: 0.1};
    let mut ray_connector = RayConnector::new(5., pos, true);
    let mut control = RayConnector::new(5., pos, true);
    let mut ray_connector_far = RayConnector::new(5., pos, false);
    let mut control_far = RayConnector::new(5., pos, false);
    for i in 0..NR_TESTS {
        let angle = i as f32 / NR_TESTS as f32 * std::f32::consts::TAU;
        let observer_pos = Vec3{x: 7. * angle.cos(), y: 7. * angle.sin(), z: 0.};
        let output = ray_connector.update_ray(observer_pos, 1);
        let output2 = control.update_ray(observer_pos, 5);
        let output_far = ray_connector_far.update_ray(observer_pos, 1);
        let output2_far = control_far.update_ray(observer_pos, 5);
        let error = (output2[3] - output[3]).abs();
        let error_far = (output2_far[3] - output_far[3]).abs();
        let angles_match = error < 5e-4 as f32; //error needs to be smaller than one pixel on a 4k display
        let angles_match_far = error_far < 5e-4 as f32;
        if angles_match {
            counter += 1;
        } 
        else {
            println!("Failed with error {error} at angle {angle}");
        }
        if angles_match_far {
            counter += 1;
        } 
        else {
            println!("Failed with error {error} at angle {angle} for the farside ray");
        }
    }
    assert_eq!(counter, 2 * NR_TESTS);


    Ok(())
}
