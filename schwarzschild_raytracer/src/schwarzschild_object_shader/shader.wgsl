//! A shader for rendering points around a black hole
const M_PI_2: f32 = 1.57079632679489661923;
// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
}

// This gets updated constantly
struct AngleInput {
    @location(3) angle: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
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
@group(2) @binding(0)
var<uniform> model_matrix: mat4x4<f32>;

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
fn vs_main(vertex: VertexInput, angle: AngleInput) -> VertexOutput {
    //apply the model transformation
    var carthesic = vec4<f32>(vertex.position, 1.);
    carthesic = model_matrix * carthesic;
    carthesic.w = 0.;

    //Project onto normal plane
    carthesic = carthesic * observer.central_to_uv;
    //Calculated traveled arc around BH for occlusion as a fraction of a whole orbit
    var bh_arc = acos(carthesic.z / length(carthesic)) / ( 4. * M_PI_2);

    // Create actual position on observer tangent space
    var polar = vec2<f32>(atan2(carthesic.y, carthesic.x), angle.angle);
    // Negative incoming angles indicate the farside ray,
    // Thus we transform to standard polar coordinates
    if polar.y < 0. {
        polar.x += 2. * M_PI_2;
        bh_arc = 1. - bh_arc;
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
    // The depth information needs to be included here
    out.clip_position = vec4<f32>(-carthesic.y, -carthesic.x, carthesic.z * bh_arc, abs(carthesic.z));
    //out.clip_position = vec4<f32>(-carthesic.y / abs(carthesic.z), -carthesic.x / abs(carthesic.z), sign(carthesic.z), 1.);
    out.tex_coords = vertex.tex_coords;
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
