//! Some utility functions to help with polar coordinates and 
//! construct matrices suitable for the coordinate conventions in this framework
//! theta = 0 is the equator in this setting
//! I mean, what lunatic seriously thinks y is up?

use glam::*;

pub fn carthesic_to_polar(vec: DVec3) -> DVec3 {
    let mut polar = DVec3::ZERO;
    polar.x = vec.length();
    if polar.x != 0. {
        polar.y = f64::atan2(vec.y, vec.x);
        polar.z = (vec.z/polar.x).asin();
    }
    return polar;
}

// (r, phi, theta) -> (x,y,z)
pub fn polar_to_carthesic(polar: DVec3) -> DVec3 {
    let mut vec = DVec3::ZERO;
    vec.x = polar.x * polar.y.cos() * polar.z.cos();
    vec.y = polar.x * polar.y.sin() * polar.z.cos();
    vec.z = polar.x * polar.z.sin();
    return vec;
}

// (phi,theta) -> (x,y,z)
pub fn polar2_to_carthesic(polar: DVec2) -> DVec3 {
    let mut vec = DVec3::ZERO;
    vec.x = polar.x.cos() * polar.y.cos();
    vec.y = polar.x.sin() * polar.y.cos();
    vec.z = polar.y.sin();
    return vec;
}

//Transforms a vector in polar coordinates according to a matrix
pub fn trans_polar_vec(polar: DVec3, trans: DMat3 ) -> DVec3 {
    let mut result = polar;
    result = polar_to_carthesic(result);
    result = trans * result;
    return carthesic_to_polar(result);
}

// Creates an rotation transformation, where z will be oriented towards the input, 
// x is rotated down from that and y points to the left.
pub fn look_to_vec_mat(look_to: DVec3) -> DMat3 {
    let z = look_to.normalize();
    let mut x_polar = carthesic_to_polar(z);
    x_polar.z -= std::f64::consts::FRAC_PI_2;
    let x = polar_to_carthesic(x_polar);
    let y = z.cross(x);

    return DMat3::from_cols(x, y, z);
}