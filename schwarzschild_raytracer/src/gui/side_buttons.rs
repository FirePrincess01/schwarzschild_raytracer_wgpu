//! the buttons on the right bottom side (excluding the menu button)

use wgpu_renderer::{gui, vertex_texture_shader::VertexTextureShaderDraw};

use super::utils::{create_rectangle_vertices, create_rectangle_indices, update_instance, create_texture};

#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub enum SideButtonId 
{
    Reset,
    Still,
    FrozenFall,
    Fall,
    Orbit,
    PerformanceMonitor,
}

pub struct SideButtons
{
    placement: gui::Gui<SideButtonId>,

    mesh_reset: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_still: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_frozen_fall: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_fall: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_orbit: wgpu_renderer::vertex_texture_shader::Mesh,
    mesh_performance_monitor: wgpu_renderer::vertex_texture_shader::Mesh,

    textures: Vec<wgpu_renderer::vertex_texture_shader::Texture>,
}

impl SideButtons {
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
            gui::GuiElement::Rectangle(gui::Rectangle::new_btn(SideButtonId::Reset,
                btn_width, btn_height, btn_boarder)),
            gui::GuiElement::Rectangle(gui::Rectangle::new_btn(SideButtonId::Still,
                btn_width, btn_height, btn_boarder)),
            gui::GuiElement::Rectangle(gui::Rectangle::new_btn(SideButtonId::FrozenFall,
                btn_width, btn_height, btn_boarder)),
            gui::GuiElement::Rectangle(gui::Rectangle::new_btn(SideButtonId::Fall,
                btn_width, btn_height, btn_boarder)),
            gui::GuiElement::Rectangle(gui::Rectangle::new_btn(SideButtonId::Orbit,
                btn_width, btn_height, btn_boarder)),
            gui::GuiElement::Rectangle(gui::Rectangle::new_btn(SideButtonId::PerformanceMonitor,
                btn_width, btn_height, btn_boarder)),
        ]);

        let placement = gui::Gui::new(width,
            height,
            vec![
                gui::AlignedElement::new(
                    gui::Alignment::BottomRight, 
                    10, 
                    10 + btn_height + 2*btn_boarder, 
                    gui::GuiElement::VerticalLayout(vertical_layout))
                ]
            );

        // meshes
        let vertices = create_rectangle_vertices(btn_width, btn_height);
        let indices = create_rectangle_indices();
        let instance = wgpu_renderer::vertex_texture_shader::Instance::zero();

        let mesh_reset = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            0, 
            &indices, 
            &[instance]);

        let mesh_still = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            1, 
            &indices, 
            &[instance]);

        let mesh_frozen_fall = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            2, 
            &indices, 
            &[instance]);

        let mesh_fall = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            3, 
            &indices, 
            &[instance]);

        let mesh_orbit = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            4, 
            &indices, 
            &[instance]);

        let mesh_performance_monitor = wgpu_renderer::vertex_texture_shader::Mesh::new(
            wgpu_renderer.device(), 
            &vertices, 
            5, 
            &indices, 
            &[instance]);

        let textures = vec![
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/reset.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/still_mode.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/paused_falling_mode.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/falling_mode.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/orbit_mode.png")),
            create_texture(wgpu_renderer, &texture_bind_group_layout, include_bytes!("assets/performance.png")),
        ];

        let mut obj = Self {
            placement,

            mesh_reset,
            mesh_still,
            mesh_frozen_fall,
            mesh_fall,
            mesh_orbit,
            mesh_performance_monitor,

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
                SideButtonId::Reset => update_instance(queue, &mut self.mesh_reset, event.x, event.y),
                SideButtonId::Still => update_instance(queue, &mut self.mesh_still, event.x, event.y),
                SideButtonId::FrozenFall => update_instance(queue, &mut self.mesh_frozen_fall, event.x, event.y),
                SideButtonId::Fall => update_instance(queue, &mut self.mesh_fall, event.x, event.y),
                SideButtonId::Orbit => update_instance(queue, &mut self.mesh_orbit, event.x, event.y),
                SideButtonId::PerformanceMonitor => update_instance(queue, &mut self.mesh_performance_monitor, event.x, event.y),
            }
        }
    }

    pub fn mouse_event(&mut self,  mouse_event: gui::MouseEvent) 
        -> (bool, Option<gui::RectanglePressedEvent<SideButtonId>>)
    {
        self.placement.mouse_event(mouse_event)
    }
}

impl VertexTextureShaderDraw for SideButtons
{
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.mesh_reset.draw(render_pass, &self.textures);
        self.mesh_still.draw(render_pass, &self.textures);
        self.mesh_frozen_fall.draw(render_pass, &self.textures);
        self.mesh_fall.draw(render_pass, &self.textures);
        self.mesh_orbit.draw(render_pass, &self.textures);
        self.mesh_performance_monitor.draw(render_pass, &self.textures);
    }
}