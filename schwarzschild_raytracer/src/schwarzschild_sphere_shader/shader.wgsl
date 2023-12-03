//! A shader for rendering spheres around a black hole

// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pos: vec3<f32>,    //needed because clip_position is in pixels in the fs
}

//This shader just covers the screen or whatever the vertices define
@vertex 
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vertex.position, 1.);
    out.pos = vertex.position;
    return out;
}

// Fragment shader
const M_PI_2: f32 = 1.57079632679489661923;

// The transformation pipeline for the observer
struct ObserverTransformations {
    screen_to_movement: mat4x4<f32>,
    movement_to_central: mat4x4<f32>,
    central_to_uv: mat4x4<f32>,
    psi_factor: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> observer: ObserverTransformations;

// The ray fan interpolation for the sphere
@group(1) @binding(0)
var ray_fan: texture_1d<f32>;

// The graphical texture of the sphere
@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //Swap to coordinate system with z facing forward, x down and y left
    var carthesic = vec4<f32>(-in.pos.y, -in.pos.x, 1., 0.);

    //Performs FOV scaling and rotates into the direction of movement
    carthesic = carthesic * observer.screen_to_movement.w;
    carthesic = normalize(observer.screen_to_movement * carthesic);
    var polar = to_polar(carthesic);

    //Special relativistic velocity abberation
    //This makes the things we move towards appear further away
    let sin_result: f32 = sin(polar.y);
    polar.y = asin((sin_result - observer.psi_factor.x) / (1. - sin_result * observer.psi_factor.x));

    carthesic = to_cart(polar);
    //Rotate to look (z) directly at the black hole
    carthesic = observer.movement_to_central * carthesic; 
    polar = to_polar(carthesic);

    //Normalizing theta to [0,1] and casting the rays onto the sphere
    polar.y = clamp((M_PI_2 - polar.y) / (M_PI_2 * 2.), 0., 1.);
    let size = u32(textureDimensions(ray_fan));
    polar.y = polar.y * f32(size - 1u);
    let index = u32(floor(polar.y));
    let weight = fract(polar.y);
    // The higher index might be invalid, but in that case weight is 0
    polar.y = textureLoad(ray_fan, index, 0).x * (1. - weight) + textureLoad(ray_fan, index + 1u, 0).x * weight;

    // Hit the black hole (or too many rotations)
    // This helps us keep the edge of the black hole more steady
    let hit_black_hole = polar.y < -7.;

    carthesic = to_cart(polar);
    // rotate to align with the texture coordinates
    carthesic = observer.central_to_uv * carthesic;
    polar = to_polar(carthesic);

    // Normalizing polar coordinates to [0,1]^2
    polar.x = polar.x / (M_PI_2 * 4.);
    if polar.x < 0. {
        polar.x += 1.;
    }
    polar.y = 0.5 - polar.y / (M_PI_2 * 2.);
    let result = textureSample(t_diffuse, s_diffuse, polar);
    if hit_black_hole {
        discard;
    }
    return result;
}
