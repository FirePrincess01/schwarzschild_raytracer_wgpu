//! A bind group to create observer uniform buffers for this shader
//!
#[allow(dead_code)]
pub struct SphereObserverBindGroupLayout {
    sphere_observer_bind_group_layout: wgpu::BindGroupLayout,
}

#[allow(dead_code)]
impl SphereObserverBindGroupLayout {

    pub fn new(device: &wgpu::Device) -> Self {

            // Camera
        let sphere_observer_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset:false, 
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("sphere_observer_bind_group_layout"),
        });

        Self {
            sphere_observer_bind_group_layout,
        }
    }

    pub fn get(&self) -> &wgpu::BindGroupLayout {
        &self.sphere_observer_bind_group_layout
    }

}