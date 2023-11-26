//! some general functions

use wgpu_renderer::vertex_texture_shader;

pub fn create_rectangle_vertices(width: u32, height: u32)
-> [vertex_texture_shader::Vertex; 4]
{
    let width = width as f32;
    let height = height as f32;

    let vertices: [vertex_texture_shader::Vertex; 4] = [
        vertex_texture_shader::Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 1.0] }, // A
        vertex_texture_shader::Vertex { position: [width, 0.0, 0.0], tex_coords: [1.0, 1.0] }, // B
        vertex_texture_shader::Vertex { position: [width, height, 0.0], tex_coords: [1.0, 0.0] }, // C
        vertex_texture_shader::Vertex { position: [0.0, height, 0.0], tex_coords: [0.0, 0.0] }, // D
    ];

    vertices
}

pub fn create_rectangle_indices() -> [u32; 6]
{
    const INDICES: [u32;6] = [
        0, 1, 2,
        2, 3, 0,
    ];

    INDICES
}

pub fn create_texture(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface,
    texture_bind_group_layout: &vertex_texture_shader::TextureBindGroupLayout,
    texture_bytes: &[u8],
) -> wgpu_renderer::vertex_texture_shader::Texture
{
    let texture_image = image::load_from_memory(texture_bytes).unwrap();
    let texture_rgba = texture_image.to_rgba8();

    create_texture_rgba(wgpu_renderer, 
        &texture_bind_group_layout, 
        &texture_rgba)
}

pub fn create_texture_rgba(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface,
    texture_bind_group_layout: &vertex_texture_shader::TextureBindGroupLayout,
    texture_rgba: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
) -> wgpu_renderer::vertex_texture_shader::Texture
{
    let texture = vertex_texture_shader::Texture::new(
        wgpu_renderer, 
        &texture_bind_group_layout, 
        &texture_rgba, 
        Some("gui texture")).unwrap(); 

    texture
}


pub fn update_instance(queue: &wgpu::Queue, 
    mesh: &mut wgpu_renderer::vertex_texture_shader::Mesh,
    x: u32, y: u32)
{
    let mut instance = vertex_texture_shader::Instance::zero();

    instance.position.x = x as f32;
    instance.position.y = y as f32;
    instance.position.z = 0.0;

    mesh.update_instance_buffer(queue, &[instance]);
}

