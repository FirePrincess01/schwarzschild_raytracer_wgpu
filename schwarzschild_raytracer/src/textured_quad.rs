use wgpu_renderer::renderer::WgpuRendererInterface;
use wgpu_renderer::vertex_texture_shader::{
    Vertex,
    Instance,
    VertexBuffer,
    IndexBuffer,
    Texture,
    InstanceBuffer, 
    VertexTextureShaderDraw, 
    TextureBindGroupLayout,
};

use image;


pub struct TexturedQuad {
    // host data
    _texture_rgba: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    _instance: Instance,

    // device data
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    texture: Texture,
    instance_buffer: InstanceBuffer,
}

impl TexturedQuad {
    pub fn new(wgpu_renderer: &mut impl WgpuRendererInterface, 
        texture_bind_group_layout: &TextureBindGroupLayout,
    ) -> Self 
    {
        let texture_image = image::load_from_memory(include_bytes!("performance.png")).unwrap();
        let texture_rgba = texture_image.to_rgba8();

        let width = texture_rgba.width();
        let height = texture_rgba.height();

        let vertex_buffer = VertexBuffer::new(wgpu_renderer.device(), 
            &Self::vertices(width, height));
        let index_buffer = IndexBuffer::new(wgpu_renderer.device(), &Self::indices());

        let texture = Texture::new(
            wgpu_renderer, 
            &texture_bind_group_layout, 
            &texture_rgba, 
            Some("example texture")).unwrap(); 

        let instance = Instance::zero();
        let instance_raw = instance.to_raw();
        let instance_buffer = InstanceBuffer::new(wgpu_renderer.device(), &[instance_raw]);
    
        Self {
            _texture_rgba: texture_rgba,
            _instance: instance,

            vertex_buffer,
            index_buffer,
            texture,
            instance_buffer,
        }
    }


    fn vertices(width: u32, height: u32) -> [Vertex; 4]
    {
        let width = width as f32;
        let height = height as f32;

        let vertices: [Vertex; 4] = [
            Vertex { position: [0.0, 0.0, 0.0], tex_coords: [0.0, 1.0] }, // A
            Vertex { position: [width, 0.0, 0.0], tex_coords: [1.0, 1.0] }, // B
            Vertex { position: [width, height, 0.0], tex_coords: [1.0, 0.0] }, // C
            Vertex { position: [0.0, height, 0.0], tex_coords: [0.0, 0.0] }, // D
        ];

        vertices
    }

    fn indices() -> [u32; 6]
    {
        const INDICES: [u32;6] = [
            0, 1, 2,
            2, 3, 0,
        ];

        INDICES
    }

    pub fn _update_texture(&mut self, queue: &wgpu::Queue) 
    {
        self.texture.write(queue, &self._texture_rgba);
    }

    pub fn _update_instance_buffer(&mut self, queue: &wgpu::Queue)
    {
        let instance_raw = self._instance.to_raw();
        self.instance_buffer.update(queue, &[instance_raw]);
    }

}

impl VertexTextureShaderDraw for TexturedQuad {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) 
    {
        self.vertex_buffer.bind(render_pass);
        self.texture.bind(render_pass);
        self.index_buffer.bind(render_pass);
        self.instance_buffer.bind_slot(render_pass, 1);

        render_pass.draw_indexed(0..self.index_buffer.size(), 0, 0..self.instance_buffer.size());
    }
}

