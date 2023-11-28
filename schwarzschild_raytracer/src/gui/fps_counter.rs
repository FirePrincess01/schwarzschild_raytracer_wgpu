//! Displays the fps

use wgpu_renderer::{gui::{self, NoId}, vertex_texture_shader::VertexTextureShaderDraw};

use super::utils::{create_rectangle_vertices, create_rectangle_indices, update_instance, create_texture_rgba};

#[derive(Copy, Clone)]
pub enum FpsCounterId 
{
    Fps,
}

pub struct  FpsCounter
{
    label_fps: wgpu_renderer::label::Label,

    placement: gui::Gui<FpsCounterId, gui::NoId, FpsCounterId>,

    mesh_fps: wgpu_renderer::vertex_texture_shader::Mesh,

    textures: Vec<wgpu_renderer::vertex_texture_shader::Texture>,
}

impl FpsCounter {
    pub fn new(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        texture_bind_group_layout: &wgpu_renderer::vertex_texture_shader::TextureBindGroupLayout,
        width: u32, 
        height: u32,
        font: &rusttype::Font) -> Self
    {
        let btn_boarder = 2;

        let label_fps = wgpu_renderer::label::Label::new(
            &font, 20.0, "1000 fps"
        );

        // placement
        let horizontal_layout1 = gui::HorizontalLayout::new(vec![
            gui::Rectangle::new(FpsCounterId::Fps, 
                label_fps.width(), label_fps.height(), btn_boarder).into(),
                ]);


        let placement = gui::Gui::new(width,
            height,
            vec![
                gui::AlignedElement::new(
                    gui::Alignment::TopLeft, 
                    10, 
                    10, 
                    horizontal_layout1.into())
                ]
            );

        // meshes
        let indices = create_rectangle_indices();
        let instance = wgpu_renderer::vertex_texture_shader::Instance::zero();

        let mesh_fps = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &create_rectangle_vertices(label_fps.width(), label_fps.height()), 
            0, 
            &indices, 
            &[instance]);

        let textures = vec![
            create_texture_rgba(wgpu_renderer, &texture_bind_group_layout, label_fps.get_image()),
        ];

        let mut obj = Self {
            label_fps,

            placement,

            mesh_fps,

            textures,
        };

        obj.resize(wgpu_renderer.queue(), width, height);

        obj
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32)
    {
        let events = self.placement.resize(width, height);
    
        for event in events {
            match event.element_id
            {
                FpsCounterId::Fps => update_instance(queue, &mut self.mesh_fps, event.x, event.y),
            }
        }
    }

    pub fn _mouse_event(&mut self,  mouse_event: gui::MouseEvent) 
        -> gui::MouseEventResult<NoId, FpsCounterId>
    {
        let mouse_res = self.placement.mouse_event(mouse_event);
        mouse_res
    }

    pub fn set_value<'a>(&mut self, 
        wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        font: &'a rusttype::Font, 
        value: u32) 
    {
        let text = value.to_string() + " fps";
        self.label_fps.update(font, &text);
        self.textures[0].write(wgpu_renderer.queue(), self.label_fps.get_image());
    }
}

impl VertexTextureShaderDraw for  FpsCounter
{
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.mesh_fps.draw(render_pass, &self.textures);
    }
}