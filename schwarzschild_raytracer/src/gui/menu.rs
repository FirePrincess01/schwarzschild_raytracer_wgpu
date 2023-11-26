//! The menu button

use wgpu_renderer::{gui, vertex_texture_shader::VertexTextureShaderDraw};

use super::utils::{create_rectangle_vertices, create_rectangle_indices, update_instance, create_texture};

#[derive(Copy, Clone)]
pub enum MenuId 
{
    Menu,
}

pub struct Menu
{
    placement: gui::Gui<MenuId>,

    mesh_menu: wgpu_renderer::vertex_texture_shader::Mesh,

    textures: Vec<wgpu_renderer::vertex_texture_shader::Texture>,
}

impl Menu {
    pub fn new(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        texture_bind_group_layout: &wgpu_renderer::vertex_texture_shader::TextureBindGroupLayout,
        width: u32, 
        height: u32) -> Self
    {
        let btn_width = 40;
        let btn_height = 40;
        let btn_boarder = 5;

        // placement
        let vertical_layout =  gui::VerticalLayout::new(vec![
            gui::GuiElement::Rectangle(gui::Rectangle::new_btn(MenuId::Menu,
                btn_width, btn_height, btn_boarder)),
        ]);

        let placement = gui::Gui::new(width,
            height,
            vec![
                gui::AlignedElement::new(
                    gui::Alignment::BottomRight, 
                    10, 
                    10, 
                    gui::GuiElement::VerticalLayout(vertical_layout))
                ]
            );

        // meshes
        let vertices = create_rectangle_vertices(btn_width, btn_height);
        let indices = create_rectangle_indices();
        let instance = wgpu_renderer::vertex_texture_shader::Instance::zero();

        let mesh_menu = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            0, 
            &indices, 
            &[instance]);

        let textures = vec![
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/menu.png")),
        ];

        let mut obj = Self {
            placement,

            mesh_menu,

            textures,
        };

        obj.resize(wgpu_renderer.queue(), width, height);

        obj
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32)
    {
        let events = self.placement.resize(width, height);
    
        for event in events {
            match event.rectangle_id
            {
                MenuId::Menu => update_instance(queue, &mut self.mesh_menu, event.x, event.y),
            }
        }
    }

    pub fn mouse_event(&mut self,  mouse_event: gui::MouseEvent) 
        -> (bool, Option<gui::RectanglePressedEvent<MenuId>>)
    {
        self.placement.mouse_event(mouse_event)
    }
}

impl VertexTextureShaderDraw for Menu
{
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.mesh_menu.draw(render_pass, &self.textures);
    }
}