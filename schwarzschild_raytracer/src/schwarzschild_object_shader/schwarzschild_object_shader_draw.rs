// interface to draw a sphere object

pub trait SchwarzschildObjectShaderDraw
{
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}