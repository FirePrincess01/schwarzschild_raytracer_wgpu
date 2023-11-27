//! This module simulates relativistic data on the CPU
//! It contains the observer, which is a more complex camera, which is responsible for screen scaling, 
//! three rotations and a nonlinear special relativistic aberration transformation
//! Furthermore it contains the tool to calculate a ray fan between an observer and a given sphere
pub mod observer;
pub mod orbit;
mod polar_transformations;
pub mod sphere_ray_tracer;

#[cfg(test)]
mod tests;