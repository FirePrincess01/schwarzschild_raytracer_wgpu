use wgpu_renderer::{performance_monitor, vertex_color_shader::{self, VertexColorShaderDraw}, renderer::WgpuRendererInterface};

const WATCHPOINTS_SIZE: usize  = 5;

pub struct PerformanceMonitor 
{
    pub watch: performance_monitor::Watch<5>,
    graph_host: performance_monitor::Graph,
    graph_device: vertex_color_shader::Mesh,

    pub show: bool,
}

impl PerformanceMonitor {
    pub fn new(wgpu_renderer: &mut impl WgpuRendererInterface) -> Self 
    {
        
        let watch: performance_monitor::Watch<WATCHPOINTS_SIZE> = performance_monitor::Watch::new(); 
        let graph_host = performance_monitor::Graph::new(WATCHPOINTS_SIZE);
        let graph_instance = vertex_color_shader::Instance{
            position: glam::Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
        };
        let graph_instances = [graph_instance];
        let graph_device = vertex_color_shader::Mesh::new(
            wgpu_renderer.device(),
            graph_host.vertices.as_slice(),
            graph_host.colors.as_slice(),
            graph_host.indices.as_slice(),
            &graph_instances,
        );

        Self {
            watch,
            graph_host,
            graph_device,

            show: true,
        }
    }

    pub fn update(&mut self, wgpu_renderer: &mut impl WgpuRendererInterface)
    {
        self.watch.update();
        self.watch.update_viewer(&mut self.graph_host);
        self.graph_device.update_vertex_buffer(wgpu_renderer.queue(), self.graph_host.vertices.as_slice());
    }
}

impl VertexColorShaderDraw for PerformanceMonitor {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.show{
            self.graph_device.draw(render_pass);
        }
    }
}