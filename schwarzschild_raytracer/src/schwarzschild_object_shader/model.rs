//! Contains a complete description of an object composed of several meshes and materials

use glam::Vec3;
use wgpu_renderer::vertex_texture_shader::Texture;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
    pub positions: Vec<Vec<Vec3>>,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: Texture,
    //pub bind_group: wgpu::BindGroup,  //contained in Texture
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

impl Mesh {
    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    }
}