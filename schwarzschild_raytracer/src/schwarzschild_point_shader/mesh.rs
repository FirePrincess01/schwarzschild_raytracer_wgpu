use wgpu_renderer::vertex_texture_shader::IndexBuffer;

use super::{vertex_buffer::VertexBuffer, vertex::Vertex};



pub struct Mesh {
    vertex_buffer: VertexBuffer,
    index_buffer: Option<IndexBuffer>,
}

impl Mesh {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex], indices: Option<&[u32]>) -> Self {
        let vertex_buffer = VertexBuffer::new(device, vertices);
        let index_buffer = match indices{
            Some(index) => {
                Some(IndexBuffer::new(device, index))
            },
            None => { None },
        };
        
        
        
        Self { vertex_buffer, index_buffer } 
    }

    pub fn update_vertex_buffer(&mut self, queue: &wgpu::Queue, vertices: &[Vertex])
    {   
        self.vertex_buffer.update(queue, vertices);
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.vertex_buffer.bind(render_pass);
        match &self.index_buffer {
            Some(buffer) => {
                buffer.bind(render_pass);
                render_pass.draw_indexed(0..buffer.size(), 0, 0..1);
            },
            None => {
                render_pass.draw(0..(self.vertex_buffer.size() as u32), 0..1);
            },
        }
    }
}