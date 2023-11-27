//! Represents a intransparent basic sphere
//! Contains the graphical surface texture and the storage texture for the corresponding ray fan.
//! Further contains the simulation tool to calculate said ray fan.

use std::f64::consts::PI;

use image::DynamicImage;
use wgpu_renderer::{vertex_texture_shader::{Texture, IndexBuffer, TextureBindGroupLayout}, vertex_color_shader::{Vertex, VertexBuffer}, renderer::WgpuRendererInterface};

use crate::{schwarzschild_sphere_shader::{ray_fan_texture::RayFanTexture, ray_fan_bind_group_layout::RayFanBindGroupLayout, schwarzschild_sphere_shader_draw::SchwarzschildSphereShaderDraw}, simulation::sphere_ray_tracer::SphereRayTracer};

pub struct BasicSphereBuffer{
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    texture: Texture,
    ray_fan: RayFanTexture,
    ray_tracer: SphereRayTracer,
}

impl BasicSphereBuffer {
    pub fn new(wgpu_renderer: &mut impl WgpuRendererInterface, 
        texture_bind_group_layout: &TextureBindGroupLayout,
        ray_fan_bind_group_layout: &RayFanBindGroupLayout,
        sphere_radius: f64,
        schwarz_radius: f64,
        texture_image: &DynamicImage,
    ) -> Self{


        //let texture_image = image::load_from_memory(include_bytes!(image_name)).unwrap();
        let texture_rgba = texture_image.to_rgba8();

        let texture = Texture::new(
            wgpu_renderer, 
            &texture_bind_group_layout, 
            &texture_rgba, 
            Some(&("Sphere r".to_owned() + &sphere_radius.to_string() + " texture"))).unwrap();

        let vertex_buffer = VertexBuffer::new(wgpu_renderer.device(), 
            &Self::vertices());
        let index_buffer = IndexBuffer::new(wgpu_renderer.device(), &Self::indices());

        let nr_nodes_half = 100;
        let ray_fan = RayFanTexture::new(wgpu_renderer, 
            ray_fan_bind_group_layout, 
            2 * nr_nodes_half as u32, 
            Some(&("Ray fan r".to_owned() + &sphere_radius.to_string())));
        let ray_tracer = SphereRayTracer::new(sphere_radius,
            schwarz_radius,
            1000, 
            PI / 100., 
            nr_nodes_half);
        
        Self {
            vertex_buffer,
            index_buffer,
            texture,
            ray_fan,
            ray_tracer,
        }
    }

    // We just cover the whole screen
    fn vertices() -> [Vertex; 4]
    {
        let vertices: [Vertex; 4] = [
            Vertex { position: [-1., -1., 0.1] }, // A
            Vertex { position: [1., -1., 0.1] }, // B
            Vertex { position: [1., 1., 0.1] }, // C
            Vertex { position: [-1., 1., 0.1] }, // D
        ];

        vertices
    }

    fn indices() -> [u32; 6]
    {
        const INDICES: [u32;6] = [
            0, 1, 2,
            2, 3, 0,
        ];

        INDICES
    }

    pub fn update_ray_fan(&mut self, queue: &wgpu::Queue, radial_position: f64) {
        let rays = self.ray_tracer.solve_ray_fan(radial_position);
        self.ray_fan.update(queue, rays);
    }

}

impl SchwarzschildSphereShaderDraw for BasicSphereBuffer {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.vertex_buffer.bind(render_pass);
        self.index_buffer.bind(render_pass);
        //self.texture.bind(render_pass);
        render_pass.set_bind_group(2, &self.texture.bind_group, &[]);
        self.ray_fan.bind(render_pass);

        render_pass.draw_indexed(0..self.index_buffer.size(), 0, 0..1);
    }
}