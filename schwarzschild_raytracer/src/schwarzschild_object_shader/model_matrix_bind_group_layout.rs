//! A bind group to create a buffer for the model matrix uniform in this shader
//!
#[allow(dead_code)]
pub struct ModelMatrixBindGroupLayout {
    model_matrix_bind_group_layout: wgpu::BindGroupLayout,
}

#[allow(dead_code)]
impl ModelMatrixBindGroupLayout {

    pub fn new(device: &wgpu::Device) -> Self {

            // Camera
        let model_matrix_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset:false, 
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("model_matrix_bind_group_layout"),
        });

        Self {
            model_matrix_bind_group_layout,
        }
    }

    pub fn get(&self) -> &wgpu::BindGroupLayout {
        &self.model_matrix_bind_group_layout
    }

}