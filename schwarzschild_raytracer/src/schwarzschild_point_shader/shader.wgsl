//! A shader for rendering points around a black hole
const M_PI_2: f32 = 1.57079632679489661923;
// Vertex shader
struct VertexInput {
    @location(0) position: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

// The transformation pipeline for the observer
struct ObserverTransformations {
    screen_to_movement: mat4x4<f32>,
    movement_to_central: mat4x4<f32>,
    central_to_uv: mat4x4<f32>,
    psi_factor: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> observer: ObserverTransformations;

// Transforms polar to carthesic coordinates
// Polar coordinates are [0, 2pi]x[-pi/2, pi/2]
fn to_cart(pVec: vec2<f32>) -> vec4<f32> {
    return vec4<f32>(cos(pVec.x)*cos(pVec.y), sin(pVec.x)*cos(pVec.y), sin(pVec.y), 0.);
}

// Transforms carthesic to polar coordinates
// input needs to be normalized
fn to_polar(cartVec: vec4<f32>) -> vec2<f32> {
    return vec2<f32>(atan2(cartVec.y, cartVec.x), asin(cartVec.z));
}

//This shader transforms the vertices backwards through the pipeline onto the screen
@vertex 
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var carthesic = vertex.position;
    //Project onto normal plane
    carthesic = carthesic * observer.central_to_uv;

    // Create actual position on observer tangent space
    var polar = vec2<f32>(atan2(carthesic.y, carthesic.x), carthesic.w);
    // Negative incoming angles indicate the farside ray,
    // Thus we transform to standard polar coordinates
    if polar.y < 0. {
        polar.x += 2. * M_PI_2;
    }
    polar.y = M_PI_2 - abs(polar.y);

    carthesic = to_cart(polar);
    carthesic = carthesic * observer.movement_to_central;
    polar = to_polar(carthesic);

    // Special relativistic velocity abberation
    // This makes the things we move towards appear further away
    // Note the inverted theta
    let sin_result: f32 = sin(-polar.y);
    polar.y = -asin((sin_result - observer.psi_factor.x) / (1. - sin_result * observer.psi_factor.x));

    carthesic = to_cart(polar);
    carthesic = carthesic * observer.screen_to_movement;
    // Screen scaling
    carthesic = carthesic / observer.screen_to_movement.w;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(-carthesic.y, -carthesic.x, carthesic.z, abs(carthesic.z));
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1., 0., 0., 1.);
}
