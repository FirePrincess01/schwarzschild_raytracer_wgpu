//! Contains a buffer for the CameraUniform struct
//! The whole thing contains three 4x4 matrices (they are 3x3, but are inflated for byte alignment),
//! a scalar velocity factor and the 3D position packed into a 4D vector

use super::super::simulation::observer::TransformationPipeline;
use super::sphere_observer_bind_group_layout;
use wgpu::util::DeviceExt;

pub struct SphereObserverUniformBuffer{
    observer_buffer: wgpu::Buffer,
    sphere_observer_bind_group: wgpu::BindGroup,
}

impl SphereObserverUniformBuffer {
    pub fn new(device: &wgpu::Device, sphere_observer_bind_group_layout: &sphere_observer_bind_group_layout::SphereObserverBindGroupLayout) -> Self {

        let observer_uniform = TransformationPipeline::new();

        let observer_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Observer Buffer"),
                contents: bytemuck::cast_slice(&[observer_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let sphere_observer_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: sphere_observer_bind_group_layout.get(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: observer_buffer.as_entire_binding(),
                }
            ],
            label: Some("observer_bind_group"),
        });

        Self {
            observer_buffer,
            sphere_observer_bind_group,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, transformations: TransformationPipeline)
    {
        queue.write_buffer(&self.observer_buffer, 0, bytemuck::cast_slice(&[transformations]));
    }

    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>){
        render_pass.set_bind_group(0, &self.sphere_observer_bind_group, &[]);
    }

}