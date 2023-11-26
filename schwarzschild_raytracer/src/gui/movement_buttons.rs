//! The movement buttons

use wgpu_renderer::{gui, vertex_texture_shader::VertexTextureShaderDraw};

use super::utils::{create_rectangle_vertices, create_rectangle_indices, update_instance, create_texture};

#[derive(Copy, Clone)]
pub enum MovementButtonId 
{
    Up, Forward, Down,
    Left, Back, Right,
}

pub struct  MovementButtons
{
    placement: gui::Gui<MovementButtonId>,

    mesh_up: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_forward: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_down: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_left: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_back: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_right: wgpu_renderer::vertex_texture_shader::Mesh,

    textures: Vec<wgpu_renderer::vertex_texture_shader::Texture>,
}

impl  MovementButtons {
    pub fn new(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        texture_bind_group_layout: &wgpu_renderer::vertex_texture_shader::TextureBindGroupLayout,
        width: u32, 
        height: u32) -> Self
    {
        let btn_width = 40;
        let btn_height = 40;
        let btn_boarder = 5;

        // placement
        let horizontal_layout1 = gui::HorizontalLayout::new(vec![
            gui::Rectangle::new_raw(MovementButtonId::Up,
                btn_width, btn_height, btn_boarder, true, true).into(),
            gui::Rectangle::new_raw(MovementButtonId::Forward,
                btn_width, btn_height, btn_boarder, true, true).into(),
            gui::Rectangle::new_raw(MovementButtonId::Down,
                btn_width, btn_height, btn_boarder, true, true).into(),
        ]);

        let horizontal_layout2 = gui::HorizontalLayout::new(vec![
            gui::Rectangle::new_raw(MovementButtonId::Left,
                btn_width, btn_height, btn_boarder, true, true).into(),
            gui::Rectangle::new_raw(MovementButtonId::Back,
                btn_width, btn_height, btn_boarder, true, true).into(),
            gui::Rectangle::new_raw(MovementButtonId::Right,
                btn_width, btn_height, btn_boarder, true, true).into(),
        ]);

        let vertical_layout =  gui::VerticalLayout::new(vec![
            horizontal_layout1.into(),
            horizontal_layout2.into(),
        ]);

        let placement = gui::Gui::new(width,
            height,
            vec![
                gui::AlignedElement::new(
                    gui::Alignment::BottomLeft, 
                    10, 
                    10, 
                    vertical_layout.into())
                ]
            );

        // meshes
        let vertices = create_rectangle_vertices(btn_width, btn_height);
        let indices = create_rectangle_indices();
        let instance = wgpu_renderer::vertex_texture_shader::Instance::zero();

        let mesh_up = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            0, 
            &indices, 
            &[instance]);

        let mesh_forward = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            1, 
            &indices, 
            &[instance]);

        let mesh_down = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            2, 
            &indices, 
            &[instance]);

        let mesh_left = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            3, 
            &indices, 
            &[instance]);

        let mesh_back = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            4, 
            &indices, 
            &[instance]);

        let mesh_right = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            5, 
            &indices, 
            &[instance]);

        let textures = vec![
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/view.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/view.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/view.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/performance.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/mode.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/mode.png")),
        ];

        let mut obj = Self {
            placement,

            mesh_up,
            mesh_forward,
            mesh_down,
            mesh_left,
            mesh_back,
            mesh_right,

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
                MovementButtonId::Up => update_instance(queue, &mut self.mesh_up, event.x, event.y),
                MovementButtonId::Forward => update_instance(queue, &mut self.mesh_forward, event.x, event.y),
                MovementButtonId::Down => update_instance(queue, &mut self.mesh_down, event.x, event.y),
                MovementButtonId::Left => update_instance(queue, &mut self.mesh_left, event.x, event.y),
                MovementButtonId::Back => update_instance(queue, &mut self.mesh_back, event.x, event.y),
                MovementButtonId::Right => update_instance(queue, &mut self.mesh_right, event.x, event.y),
            }
        }
    }

    pub fn mouse_event(&mut self,  mouse_event: gui::MouseEvent) 
        -> (bool, Option<gui::RectanglePressedEvent<MovementButtonId>>)
    {
        self.placement.mouse_event(mouse_event)
    }
}

impl VertexTextureShaderDraw for  MovementButtons
{
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.mesh_up.draw(render_pass, &self.textures);
        self.mesh_forward.draw(render_pass, &self.textures);
        self.mesh_down.draw(render_pass, &self.textures);
        self.mesh_left.draw(render_pass, &self.textures);
        self.mesh_back.draw(render_pass, &self.textures);
        self.mesh_right.draw(render_pass, &self.textures);
    }
}