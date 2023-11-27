//! The 1D texture storing the ray fan for the shader
//! The ray fan is a interpolated function: [-pi/2, pi/2]-> (-infty, pi/2],NaN
//! representing the arc traveled by a ray shot at an input angle until it hits the surface, if it doesnt the result is NaN
//! In practice we treat any result below -10 as not hitting the sphere.


use wgpu_renderer::renderer;

use super::ray_fan_bind_group_layout::RayFanBindGroupLayout;

pub struct RayFanTexture {
    pub texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
    pub length: u32,
}

impl RayFanTexture {
    pub fn new(
        wgpu_renderer: &mut impl renderer::WgpuRendererInterface,
        ray_fan_bind_group_layout: &RayFanBindGroupLayout,
        length: u32,
        label: Option<&str>
    ) -> Self {
        let size = wgpu::Extent3d {
            width: length,
            height: 1,
            depth_or_array_layers: 1,
        };

        let texture = wgpu_renderer.device().create_texture(
            &wgpu::TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D1,
                format: wgpu::TextureFormat::R32Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = wgpu_renderer.device().create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: ray_fan_bind_group_layout.get(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ],
                label: Some("ray_fan_bind_group"),
            }
        );

        Self { 
            texture, 
            bind_group,
            length,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, ray_fan: &[f32] ) 
    {
        let size = wgpu::Extent3d {
            width: self.length,
            height: 1,
            depth_or_array_layers: 1,
        };

        let data = bytemuck::cast_slice(ray_fan);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row:  Some(4 * self.length),
                rows_per_image: Some(1),
            },
            size,
        );
    }

    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>,) {
        render_pass.set_bind_group(1, &self.bind_group, &[]);
    }

}

