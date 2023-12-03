//! GPU memory buffer containing the vertices for this shader
//!

use wgpu::util::DeviceExt;
use super::vertex::Vertex;

pub struct VertexBuffer {
    buffer: wgpu::Buffer,
    size: u32,
}

impl VertexBuffer {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex])  -> Self
    {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Self {
            buffer,
            size: vertices.len() as u32,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, vertices: &[Vertex])
    {   
        let data = bytemuck::cast_slice(vertices);

        if self.buffer.size() == data.len() as u64 {
            queue.write_buffer(&self.buffer, 0, data);
        }
    }

    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>,) 
    {
        render_pass.set_vertex_buffer(0, self.buffer.slice(..));
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}