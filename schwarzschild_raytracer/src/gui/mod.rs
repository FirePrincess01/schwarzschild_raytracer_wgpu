//! The top level gui class, contains all gui elements

use wgpu_renderer::{gui, vertex_texture_shader::VertexTextureShaderDraw};

mod adjust_spin;
mod movement_buttons;
mod side_buttons;
mod menu;
mod utils;

pub use side_buttons::SideButtonId;

pub struct Gui 
{
    width: u32,
    height: u32,

    gui_menu: menu::Menu,
    gui_side_buttons: side_buttons::SideButtons,
    gui_movement_buttons: movement_buttons::MovementButtons,

    show_side_buttons: bool,
    show_movement_buttons: bool,
    show_adjust_spin: bool,
}

impl Gui {
    pub fn new(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        texture_bind_group_layout: &wgpu_renderer::vertex_texture_shader::TextureBindGroupLayout,
        width: u32, 
        height: u32) -> Self
    {
        let gui_menu = menu::Menu::new(
            wgpu_renderer, 
            texture_bind_group_layout, 
            width, 
            height);

        let gui_side_buttons = side_buttons::SideButtons::new(
            wgpu_renderer, 
            texture_bind_group_layout, 
            width, 
            height);

        let gui_movement_buttons = movement_buttons::MovementButtons::new(
            wgpu_renderer, 
            texture_bind_group_layout, 
            width, 
            height);

        Self {
            width,
            height,

            gui_menu,
            gui_side_buttons,
            gui_movement_buttons,

            show_side_buttons: false,
            show_movement_buttons: true,
            show_adjust_spin: false,
        }
    }

    fn handle_gui_menu_event(&mut self, event: gui::RectanglePressedEvent<menu::MenuId>) {
        match event.rectangle_id {
            menu::MenuId::Menu => {
                self.show_side_buttons = !self.show_side_buttons;
            },
        }
    }
    
    pub fn resize(&mut self, wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface,
        width: u32, height: u32)
    {
        self.width = width;
        self.height = height;

        self.gui_menu.resize(wgpu_renderer.queue(), width, height);
        self.gui_side_buttons.resize(wgpu_renderer.queue(), width, height);
        self.gui_movement_buttons.resize(wgpu_renderer.queue(), width, height);
    }

    pub fn mouse_moved(&mut self, x: u32, y: u32) -> bool
    {
        // change from mouse coordinate system to the gui coordinate system
        let y = self.height - y.min(self.height);

        let mouse_event = gui::MouseEvent::Moved{ x, y };

        // menu
        let (consumed, _events) = self.gui_menu.mouse_event(mouse_event);
        if consumed {
            return true;
        }

        // side buttons
        if self.show_side_buttons {
            let (consumed, _events) = self.gui_side_buttons.mouse_event(mouse_event);
            if consumed {
                return true;
            }
        }

        // movement buttons
        if self.show_movement_buttons {

        }

        // adjust_spin
        if self.show_adjust_spin {

        }

        false
    }

    pub fn mouse_pressed(&mut self, pressed: bool) 
        -> (bool, Option<gui::RectanglePressedEvent<side_buttons::SideButtonId>>)
    {
        let mouse_event = if pressed {
            gui::MouseEvent::Pressed
        } else {
            gui::MouseEvent::Released
        };

        // menu
        let (consumed, event) = self.gui_menu.mouse_event(mouse_event);
        if consumed {
            match event {
                Some(event) => { 
                    self.handle_gui_menu_event(event);
                },
                None => {},
            }

            return (true, None);
        }

        // side buttons
        if self.show_side_buttons {
            let (consumed, event) = self.gui_side_buttons.mouse_event(mouse_event);
            if consumed {
                return (true, event);
            }
        }

        // movement buttons
        if self.show_movement_buttons {

        }

        // adjust_spin
        if self.show_adjust_spin {

        }


        (false, None)
    }
}

impl VertexTextureShaderDraw for Gui
{
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        
        // menu
        self.gui_menu.draw(render_pass);

        // side buttons
        if self.show_side_buttons {
            self.gui_side_buttons.draw(render_pass);
        }

        // movement buttons
        if self.show_movement_buttons {
            self.gui_movement_buttons.draw(render_pass);
        }
    }
}