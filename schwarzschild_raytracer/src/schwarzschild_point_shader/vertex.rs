//! The Vertex struct used in the shader
//!

use wgpu;

//Contains [x,y,z, incoming_angle]
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
}

impl Vertex {
    pub fn _zero() -> Self {
        Self { position: [0.0, 0.0, 0.0, 0.0] }
    }

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0, 
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ]
        }
    }
}