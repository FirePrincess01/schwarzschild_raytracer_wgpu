//! Contains a buffer for the Model matrix uniform


use super::model_matrix_bind_group_layout;
use wgpu::util::DeviceExt;

pub struct ModelMatrixUniformBuffer{
    model_matrix_buffer: wgpu::Buffer,
    model_matrix_bind_group: wgpu::BindGroup,
}

impl ModelMatrixUniformBuffer {
    pub fn new(device: &wgpu::Device, model_matrix_bind_group_layout: &model_matrix_bind_group_layout::ModelMatrixBindGroupLayout) -> Self {
        let model_matrix_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Observer Buffer"),
                contents: bytemuck::cast_slice(&[glam::Mat4::IDENTITY.to_cols_array()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let model_matrix_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: model_matrix_bind_group_layout.get(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: model_matrix_buffer.as_entire_binding(),
                }
            ],
            label: Some("model_matrix_bind_group"),
        });

        Self {
            model_matrix_buffer,
            model_matrix_bind_group,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, model_matrix: &glam::Mat4)
    {
        queue.write_buffer(&self.model_matrix_buffer, 0, bytemuck::cast_slice(&[model_matrix.to_cols_array()]));
    }

    pub fn bind<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>){
        render_pass.set_bind_group(2, &self.model_matrix_bind_group, &[]);
    }

}