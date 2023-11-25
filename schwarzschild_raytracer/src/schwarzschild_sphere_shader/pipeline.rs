//! A specialized shader to render spheres centered on a black hole
//!

use wgpu::BlendState;
use wgpu_renderer::renderer::depth_texture::DepthTexture;
use wgpu_renderer::vertex_color_shader::Vertex;
use wgpu_renderer::vertex_texture_shader::TextureBindGroupLayout;
use super::sphere_observer_bind_group_layout::SphereObserverBindGroupLayout;
use super::ray_fan_bind_group_layout::RayFanBindGroupLayout;


/// 
#[allow(dead_code)]
pub struct Pipeline
{
    render_pipeline: wgpu::RenderPipeline,
}

#[allow(dead_code)]
impl Pipeline
{
    pub fn new(
        device: &wgpu::Device, 
        sphere_observer_bind_group_layout: &SphereObserverBindGroupLayout,
        ray_fan_bind_group_layout: &RayFanBindGroupLayout,
        texture_bind_group_layout: &TextureBindGroupLayout, 
        surface_format: wgpu::TextureFormat,
    ) -> Self
    {
        // Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Schwarzschild Sphere Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });



        // Pipeline
        let render_pipeline_layout = 
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &sphere_observer_bind_group_layout.get(),
                    &ray_fan_bind_group_layout.get(),
                    &texture_bind_group_layout.get(),
                ],
                push_constant_ranges: &[],
            });


        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Schwarzschild Sphere Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", 
                buffers: &[
                    Vertex::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader, 
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { 
                    format: surface_format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,  // counter-clockwise direction
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill, 
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            render_pipeline,
        }
    }

    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>){
        render_pass.set_pipeline(&self.render_pipeline);
    }

}