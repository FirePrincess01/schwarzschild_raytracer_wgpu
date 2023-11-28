//! The top level gui class, contains all gui elements

use wgpu_renderer::{gui::{self, MouseEvent}, vertex_texture_shader::VertexTextureShaderDraw};

mod adjust_spin;
mod movement_buttons;
mod side_buttons;
mod menu;
mod fps_counter;
mod utils;

pub use side_buttons::SideButtonId;
pub use movement_buttons::MovementButtonId;
pub use adjust_spin::AdjustSpinButtonId;

pub enum PressedEvent {
    MovementButton(MovementButtonId),
}

pub enum ReleasedEvent {
    SideButton(SideButtonId),
    MovementButton(MovementButtonId),
    AdjustSpin(AdjustSpinButtonId),
}

pub struct GuiResult {
    pub pressed_event: Option<PressedEvent>,
    pub released_event: Option<ReleasedEvent>,
    pub consumed: bool,
}

pub struct Gui 
{
    width: u32,
    height: u32,

    gui_menu: menu::Menu,
    gui_side_buttons: side_buttons::SideButtons,
    gui_movement_buttons: movement_buttons::MovementButtons,
    gui_adjust_spin: adjust_spin::AdjustSpin,
    gui_fps_counter: fps_counter::FpsCounter,

    show_side_buttons: bool,
    show_movement_buttons: bool,
    show_adjust_spin: bool,
}

impl Gui {
    pub fn new(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        texture_bind_group_layout: &wgpu_renderer::vertex_texture_shader::TextureBindGroupLayout,
        width: u32, 
        height: u32,
        font: &rusttype::Font) -> Self
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

        let gui_adjust_spin = adjust_spin::AdjustSpin::new(
            wgpu_renderer, 
            texture_bind_group_layout, 
            width, 
            height,
            font);

        let gui_fps_counter = fps_counter::FpsCounter::new(
            wgpu_renderer, 
            texture_bind_group_layout, 
            width, 
            height,
            font);

        Self {
            width,
            height,

            gui_menu,
            gui_side_buttons,
            gui_movement_buttons,
            gui_adjust_spin,
            gui_fps_counter,

            show_side_buttons: false,
            show_movement_buttons: true,
            show_adjust_spin: false,
        }
    }

    fn handle_gui_menu_event(&mut self, event: menu::MenuId) {
        match event {
            menu::MenuId::Menu => {
                self.show_side_buttons = !self.show_side_buttons;
            },
        }
    }

    fn handle_side_button_event(&mut self, event: side_buttons::SideButtonId) {
        match event {
            SideButtonId::Orbit => {
                self.show_adjust_spin = true;
            },
            _ => {}
        }
    }

    fn handle_adjust_spin_event(&mut self, event: adjust_spin::AdjustSpinButtonId) {
        match event {
            AdjustSpinButtonId::Confirm => {
                self.show_adjust_spin = false;
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
        self.gui_adjust_spin.resize(wgpu_renderer.queue(), width, height);
        self.gui_fps_counter.resize(wgpu_renderer.queue(), width, height);
    }

    fn mouse_event(&mut self, mouse_event: MouseEvent) -> GuiResult
    {
        let mut gui_result = GuiResult{ pressed_event: None, released_event: None, consumed: false };

        // menu
        let res: gui::MouseEventResult<gui::NoId, menu::MenuId> = self.gui_menu.mouse_event(mouse_event);
        match res.released_event {
            Some(event) => self.handle_gui_menu_event(event),
            None => {},
        }
        gui_result.consumed = gui_result.consumed || res.consumed;


        // side buttons
        if self.show_side_buttons {
            let res = self.gui_side_buttons.mouse_event(mouse_event);
            match res.released_event {
                Some(event) => { 
                    self.handle_side_button_event(event);
                    gui_result.released_event = Some(ReleasedEvent::SideButton(event)); 
                },
                None => {}
            }
            gui_result.consumed = gui_result.consumed || res.consumed;
        }

        // movement buttons
        if self.show_movement_buttons {
            let res = self.gui_movement_buttons.mouse_event(mouse_event);
            match res.released_event {
                Some(event) => { gui_result.released_event = Some(ReleasedEvent::MovementButton(event)); },
                None => {}
            }
            match res.pressed_event {
                Some(event) => { gui_result.pressed_event = Some(PressedEvent::MovementButton(event)); },
                None => {}
            }
            gui_result.consumed = gui_result.consumed || res.consumed;
        }

        // adjust_spin
        if self.show_adjust_spin {
            let res = self.gui_adjust_spin.mouse_event(mouse_event);
            match res.released_event {
                Some(event) => { 
                    self.handle_adjust_spin_event(event);
                    gui_result.released_event = Some(ReleasedEvent::AdjustSpin(event)); 
                },
                None => {}
            }
            gui_result.consumed = gui_result.consumed || res.consumed;
        }

        gui_result
    }

    pub fn mouse_moved(&mut self, x: u32, y: u32) -> GuiResult
    {
        // change from mouse coordinate system to the gui coordinate system
        let y = self.height - y.min(self.height);

        let mouse_event = gui::MouseEvent::Moved{ x, y };

        self.mouse_event(mouse_event)
    }

    pub fn mouse_pressed(&mut self, pressed: bool) 
        -> GuiResult
    {
        let mouse_event = if pressed {
            gui::MouseEvent::Pressed
        } else {
            gui::MouseEvent::Released
        };

        self.mouse_event(mouse_event)
    }

    pub fn adjust_spin_set_value<'a>(&mut self, 
        wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        font: &'a rusttype::Font, 
        value: u32) 
    {
        self.gui_adjust_spin.set_value(wgpu_renderer, font, value);
    }

    pub fn adjust_spin_set_colors(&mut self, red: bool, orange: bool, green: bool) 
    {
        self.gui_adjust_spin.set_colors(red, orange, green);
    }

    pub fn fps_counter_set_value<'a>(&mut self, 
        wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        font: &'a rusttype::Font, 
        value: u32) 
    {
        self.gui_fps_counter.set_value(wgpu_renderer, font, value);
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
            self.gui_fps_counter.draw(render_pass);
        }

        // movement buttons
        if self.show_movement_buttons {
            self.gui_movement_buttons.draw(render_pass);
        }

        // adjust_spin
        if self.show_adjust_spin {
            self.gui_adjust_spin.draw(render_pass);
        }
    }
}