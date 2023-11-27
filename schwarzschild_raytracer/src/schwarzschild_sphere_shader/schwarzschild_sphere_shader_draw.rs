// interface to draw a sphere object

pub trait SchwarzschildSphereShaderDraw
{
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}