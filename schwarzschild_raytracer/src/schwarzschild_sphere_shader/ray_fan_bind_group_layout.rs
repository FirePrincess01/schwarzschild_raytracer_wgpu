//! A bind group to contain the ray fan in the shader
//! It is a 1D Float storage array without sampling
pub struct RayFanBindGroupLayout {
    ray_fan_bind_group_layout: wgpu::BindGroupLayout,
}

impl RayFanBindGroupLayout {

    pub fn new(device: &wgpu::Device) -> Self {

        // Texture
        let ray_fan_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D1,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    },
                    count: None,
                },
            ],
            label: Some("ray_fan_bind_group_layout"),
        });

        Self {
            ray_fan_bind_group_layout,
        }
    }

    pub fn get(&self) -> &wgpu::BindGroupLayout {
        &self.ray_fan_bind_group_layout
    }

}