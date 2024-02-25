// interface to draw a model around a black hole

pub trait SchwarzschildObjectShaderDraw
{
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}