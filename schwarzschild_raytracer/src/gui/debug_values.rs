//! Displays some debug values

use wgpu_renderer::{gui::{self, NoId}, vertex_texture_shader::VertexTextureShaderDraw};

use super::utils::{create_rectangle_vertices, create_rectangle_indices, update_instance, create_texture_rgba};

#[derive(Copy, Clone)]
pub enum DebugValuesId 
{
    X,
    Y,
    Z,
}

pub struct  DebugValues
{
    label_x: wgpu_renderer::label::Label,
    label_y: wgpu_renderer::label::Label,
    label_z: wgpu_renderer::label::Label,

    placement: gui::Gui<DebugValuesId, gui::NoId, DebugValuesId>,

    mesh_x: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_y: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_z: wgpu_renderer::vertex_texture_shader::Mesh,

    textures: Vec<wgpu_renderer::vertex_texture_shader::Texture>,
}

impl DebugValues {
    pub fn new(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        texture_bind_group_layout: &wgpu_renderer::vertex_texture_shader::TextureBindGroupLayout,
        width: u32, 
        height: u32,
        font: &rusttype::Font) -> Self
    {
        let btn_boarder = 2;
        let font_size: u32 = 20;

        let label_x = wgpu_renderer::label::Label::new(
            &font, font_size as f32, "x: 1000.00"
        );

        let label_y = wgpu_renderer::label::Label::new(
            &font, font_size as f32, "y: 1000.00"
        );

        let label_z = wgpu_renderer::label::Label::new(
            &font, font_size as f32, "z: 1000.00"
        );

        // placement
        let vertical_layout = gui::VerticalLayout::new(vec![
            gui::Rectangle::new(DebugValuesId::X, 
                label_x.width(), label_x.height(), btn_boarder).into(),
            gui::Rectangle::new(DebugValuesId::Y, 
                label_y.width(), label_y.height(), btn_boarder).into(),
            gui::Rectangle::new(DebugValuesId::Z, 
                label_z.width(), label_z.height(), btn_boarder).into(),
            ]);

        let placement = gui::Gui::new(width,
            height,
            vec![
                gui::AlignedElement::new(
                    gui::Alignment::TopLeft, 
                    10, 
                    20 + 2 * btn_boarder + font_size, 
                    vertical_layout.into())
                ]
            );

        // meshes
        let indices = create_rectangle_indices();
        let instance = wgpu_renderer::vertex_texture_shader::Instance::zero();

        let mesh_x = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &create_rectangle_vertices(label_x.width(), label_x.height()), 
            0, 
            &indices, 
            &[instance]);

        let mesh_y = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &create_rectangle_vertices(label_y.width(), label_y.height()), 
            1, 
            &indices, 
            &[instance]);

        let mesh_z = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &create_rectangle_vertices(label_z.width(), label_z.height()), 
            2, 
            &indices, 
            &[instance]);

        let textures = vec![
            create_texture_rgba(wgpu_renderer, &texture_bind_group_layout, label_x.get_image()),
            create_texture_rgba(wgpu_renderer, &texture_bind_group_layout, label_y.get_image()),
            create_texture_rgba(wgpu_renderer, &texture_bind_group_layout, label_z.get_image()),
        ];

        let mut obj = Self {
            label_x,
            label_y,
            label_z,

            placement,

            mesh_x,
            mesh_y,
            mesh_z,

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
                DebugValuesId::X => update_instance(queue, &mut self.mesh_x, event.x, event.y),
                DebugValuesId::Y => update_instance(queue, &mut self.mesh_y, event.x, event.y),
                DebugValuesId::Z => update_instance(queue, &mut self.mesh_z, event.x, event.y),
            }
        }
    }

    pub fn _mouse_event(&mut self,  mouse_event: gui::MouseEvent) 
        -> gui::MouseEventResult<NoId, DebugValuesId>
    {
        let mouse_res = self.placement.mouse_event(mouse_event);
        mouse_res
    }

    pub fn set_value<'a>(&mut self, 
        wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        font: &'a rusttype::Font, 
        x: f32, y: f32, z: f32) 
    {
        let text =  String::from("x: ") + &x.to_string();
        self.label_x.update(font, &text);
        self.textures[0].write(wgpu_renderer.queue(), self.label_x.get_image());

        let text =  String::from("y: ") + &y.to_string();
        self.label_y.update(font, &text);
        self.textures[1].write(wgpu_renderer.queue(), self.label_y.get_image());

        let text =  String::from("z: ") + &z.to_string();
        self.label_z.update(font, &text);
        self.textures[2].write(wgpu_renderer.queue(), self.label_z.get_image());
    }
}

impl VertexTextureShaderDraw for  DebugValues
{
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.mesh_x.draw(render_pass, &self.textures);
        self.mesh_y.draw(render_pass, &self.textures);
        self.mesh_z.draw(render_pass, &self.textures);
    }
}