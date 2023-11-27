//! This module contains the all the tools to create a working sphere shader around a black hole, which uses simulation data as input
//! Including shader, bindings, layouts, textures, the pipeline etc.

pub mod ray_fan_bind_group_layout;
pub mod ray_fan_texture;
pub mod sphere_observer_bind_group_layout;
pub mod sphere_observer_uniform_buffers;
pub mod pipeline;
pub mod schwarzschild_sphere_shader_draw;

pub mod sphere_buffer;