//! The adjust spin submenu

use wgpu_renderer::{gui::{self, NoId}, vertex_texture_shader::VertexTextureShaderDraw};

use super::utils::{create_rectangle_vertices, create_rectangle_indices, update_instance, create_texture, create_texture_rgba};

#[derive(Copy, Clone)]
pub enum AdjustSpinId 
{
    Title,
    Value,
    Red,
    Orange,
    Green,
    Confirm,
}

#[derive(Copy, Clone)]
pub enum AdjustSpinButtonId
{
    Confirm,
}

pub struct  AdjustSpin
{
    label_value: wgpu_renderer::label::Label,

    placement: gui::Gui<AdjustSpinId, gui::NoId, AdjustSpinButtonId>,

    mesh_title: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_value: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_red: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_orange: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_green: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_confirm: wgpu_renderer::vertex_texture_shader::Mesh,

    textures: Vec<wgpu_renderer::vertex_texture_shader::Texture>,
}

impl AdjustSpin {
    pub fn new(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        texture_bind_group_layout: &wgpu_renderer::vertex_texture_shader::TextureBindGroupLayout,
        width: u32, 
        height: u32,
        font: &rusttype::Font) -> Self
    {
        let btn_width = 20;
        let btn_height = 20;
        let btn_boarder = 2;

        let label_title = wgpu_renderer::label::Label::new(
            &font, 20.0, "Adjust Spin:"
        );

        let label_value = wgpu_renderer::label::Label::new(
            &font, 20.0, "100"
        );

        let label_confirm = wgpu_renderer::label::Label::new(
            &font, 20.0, "Confirm"
        );

        // placement
        let vertical_layout1 = gui::VerticalLayout::new(vec![
            gui::Rectangle::new(AdjustSpinId::Red,
                btn_width, btn_height, btn_boarder).into(),
            gui::Rectangle::new(AdjustSpinId::Orange, 
                btn_width, btn_height, btn_boarder).into(),
            gui::Rectangle::new(AdjustSpinId::Green, 
                btn_width, btn_height, btn_boarder).into(),
        ]);

        let horizontal_layout1 = gui::HorizontalLayout::new(vec![
            gui::Rectangle::new(AdjustSpinId::Title, 
                label_title.width(), label_title.height(), btn_boarder).into(),
            gui::Rectangle::new(AdjustSpinId::Value, 
                label_value.width(), label_value.height(), btn_boarder).into(),
            vertical_layout1.into(),
        ]);

        let vertical_layout2 =  gui::VerticalLayout::new(vec![
            horizontal_layout1.into(),
            gui::Rectangle::new_btn(AdjustSpinId::Confirm, AdjustSpinButtonId::Confirm,
                label_confirm.width(), label_confirm.height(), btn_boarder).into(),
        ]);

        let placement = gui::Gui::new(width,
            height,
            vec![
                gui::AlignedElement::new(
                    gui::Alignment::Center, 
                    0, 
                    0, 
                    vertical_layout2.into())
                ]
            );

        // meshes
        let vertices = create_rectangle_vertices(btn_width, btn_height);
        let indices = create_rectangle_indices();
        let instance = wgpu_renderer::vertex_texture_shader::Instance::zero();
        



        let mesh_title = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &create_rectangle_vertices(label_title.width(), label_title.height()), 
            0, 
            &indices, 
            &[instance]);

        let mesh_value = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &create_rectangle_vertices(label_value.width(), label_value.height()), 
            1, 
            &indices, 
            &[instance]);

        let mesh_red = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            2, 
            &indices, 
            &[instance]);

        let mesh_orange = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            3, 
            &indices, 
            &[instance]);

        let mesh_green = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            4, 
            &indices, 
            &[instance]);

        let mesh_confirm = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &create_rectangle_vertices(label_confirm.width(), label_confirm.height()), 
            5, 
            &indices, 
            &[instance]);

        let textures = vec![
            create_texture_rgba(wgpu_renderer, &texture_bind_group_layout, label_title.get_image()),
            create_texture_rgba(wgpu_renderer, &texture_bind_group_layout, label_value.get_image()),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/red.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/orange.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/green.png")),
            create_texture_rgba(wgpu_renderer, &texture_bind_group_layout, label_confirm.get_image()),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/grey.png")),

        ];

        let mut obj = Self {
            label_value,

            placement,

            mesh_title,
            mesh_value,
            mesh_red,
            mesh_orange,
            mesh_green,
            mesh_confirm,

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
                AdjustSpinId::Title => update_instance(queue, &mut self.mesh_title, event.x, event.y),
                AdjustSpinId::Value => update_instance(queue, &mut self.mesh_value, event.x, event.y),
                AdjustSpinId::Red => update_instance(queue, &mut self.mesh_red, event.x, event.y),
                AdjustSpinId::Orange => update_instance(queue, &mut self.mesh_orange, event.x, event.y),
                AdjustSpinId::Green => update_instance(queue, &mut self.mesh_green, event.x, event.y),
                AdjustSpinId::Confirm => update_instance(queue, &mut self.mesh_confirm, event.x, event.y),
            }
        }
    }

    pub fn mouse_event(&mut self,  mouse_event: gui::MouseEvent) 
        -> gui::MouseEventResult<NoId, AdjustSpinButtonId>
    {
        let mouse_res = self.placement.mouse_event(mouse_event);
        mouse_res
    }

    pub fn set_value<'a>(&mut self, 
        wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        font: &'a rusttype::Font, 
        value: u32) 
    {
        let text = value.to_string();
        self.label_value.update(font, &text);
        self.textures[1].write(wgpu_renderer.queue(), self.label_value.get_image());
    }

    pub fn set_colors(&mut self, red: bool, orange: bool, green: bool) 
    {
        let index_red = if red { 2 } else { 6 };
        let index_orange = if orange { 3 } else { 6 };
        let index_green = if green { 4 } else { 6 };

        self.mesh_red._set_texture_index(index_red);
        self.mesh_orange._set_texture_index(index_orange);
        self.mesh_green._set_texture_index(index_green);
    }
}

impl VertexTextureShaderDraw for  AdjustSpin
{
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.mesh_title.draw(render_pass, &self.textures);
        self.mesh_value.draw(render_pass, &self.textures);
        self.mesh_red.draw(render_pass, &self.textures);
        self.mesh_orange.draw(render_pass, &self.textures);
        self.mesh_green.draw(render_pass, &self.textures);
        self.mesh_confirm.draw(render_pass, &self.textures);
    }
}