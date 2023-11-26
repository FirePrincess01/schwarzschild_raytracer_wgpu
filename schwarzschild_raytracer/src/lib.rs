
mod renderer;
mod geometry;
mod performance_monitor;
mod textured_quad;
mod gui;

use wgpu_renderer::default_window;
use winit::event::{WindowEvent, KeyboardInput, VirtualKeyCode, ElementState, TouchPhase, MouseButton};

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;


struct SchwarzschildRaytracer {
    size: winit::dpi::PhysicalSize<u32>,
    scale_factor: f32,

    renderer: renderer::Renderer,
    performance_monitor: performance_monitor::PerformanceMonitor,

    // data
    textured_quad: textured_quad::TexturedQuad,

    // gui
    font: rusttype::Font<'static>,
    gui: gui::Gui,
}

impl SchwarzschildRaytracer {
    pub async fn new(window: &winit::window::Window) -> Self 
    {
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let width = size.width;
        let height = size.height;

        let mut renderer = renderer::Renderer::new(window).await;
        let performance_monitor = performance_monitor::PerformanceMonitor::new(
            &mut renderer.wgpu_renderer);

        // data
        let textured_quad = textured_quad::TexturedQuad::new(
            &mut renderer.wgpu_renderer, 
            &renderer.texture_bind_group_layout);
        
        // gui

        let font_data = include_bytes!("../../wgpu_renderer/src/freefont/FreeMono.ttf");
        let font = rusttype::Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font");
        let gui = gui::Gui::new(&mut renderer.wgpu_renderer, 
            &renderer.texture_bind_group_layout, 
            width, 
            height,
            &font);

        Self {
            size,
            scale_factor,

            renderer,
            performance_monitor,

            textured_quad,

            font,
            gui,
        }
    }

    fn handle_gui_event(&mut self, 
        gui_event: Option<gui::GuiEvent>)
    {
        match gui_event {
            Some(gui_event) => {
                match gui_event {
                    gui::GuiEvent::SideButton( id ) => {
                        match id {
                            gui::SideButtonId::Reset => {},
                            gui::SideButtonId::Still => {},
                            gui::SideButtonId::FrozenFall => {},
                            gui::SideButtonId::Fall => {},
                            gui::SideButtonId::Orbit => {},
                        }
                    },
                    gui::GuiEvent::MovementButton { id, pressed } => {
                        match id {
                            gui::MovementButtonId::Up => {
                                self.gui.adjust_spin_set_colors(true, false, false);
                            },
                            gui::MovementButtonId::Forward => {
                                self.gui.adjust_spin_set_colors(false, true, false);
                            },
                            gui::MovementButtonId::Down => {
                                self.gui.adjust_spin_set_colors(false, false, true);
                            },  
                            gui::MovementButtonId::Left => {
                                let value = if pressed { 10 } else { 11 };
                                self.gui.adjust_spin_set_value(&mut self.renderer.wgpu_renderer, &self.font, value);
                            },
                            gui::MovementButtonId::Back => {
                                let value = if pressed { 20 } else { 21 };
                                self.gui.adjust_spin_set_value(&mut self.renderer.wgpu_renderer, &self.font, value);
                            },
                            gui::MovementButtonId::Right => {
                                let value = if pressed { 30 } else { 31 };
                                self.gui.adjust_spin_set_value(&mut self.renderer.wgpu_renderer, &self.font, value);
                            },
                        }
                    },
                    gui::GuiEvent::AdjustSpin(id) => {
                        match id {
                            gui::AdjustSpinButtonId::Confirm => {
                                self.gui.adjust_spin_set_colors(true, true, true);
                            },
                        }
                    },
                }
            },
            None => {},
        }
    }
}

#[allow(unused)]
fn apply_scale_factor(position: winit::dpi::PhysicalPosition<f64>, scale_factor: f32) 
-> winit::dpi::PhysicalPosition<f64> 
{
    cfg_if::cfg_if! {
        // apply scale factor for the web
        if #[cfg(target_arch = "wasm32")] {
            let mut res = position;
            res.x = res.x / scale_factor as f64;
            res.y = res.y / scale_factor as f64;
            res
        }
        else {
            position
        }
    }
}

impl default_window::DefaultWindowApp for SchwarzschildRaytracer 
{
    fn get_size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        let width = new_size.width;
        let height = new_size.height;

        self.renderer.resize(new_size);
        self.gui.resize(&mut self.renderer.wgpu_renderer, width, height);
    }

    fn update_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    fn update(&mut self, dt: instant::Duration) {
        self.renderer.update(dt);

        // self.performance_monitor.watch.start(3);
        //     // update stuff
        // self.performance_monitor.watch.stop(3);
        
        // self.performance_monitor.watch.start(4);
        //     // update more stuff
        // self.performance_monitor.watch.stop(4);

        self.performance_monitor.update(&mut self.renderer.wgpu_renderer);
    }

    fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.performance_monitor.watch.start(2);
            let res = match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::F2),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => { 
                    self.performance_monitor.show = !self.performance_monitor.show;
                    true
                },
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(key),
                            state,
                            ..
                        },
                    ..
                } => self.renderer.process_keyboard(*key, *state),
                WindowEvent::MouseWheel { delta, .. } => {
                    self.renderer.process_scroll(delta);
                    true
                }
                WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state,//ElementState::Pressed,
                    ..
                } => {
                    let is_pressed = *state == ElementState::Pressed;
                    
                    let (consumed, gui_event) = self.gui.mouse_pressed(is_pressed);
                    self.handle_gui_event(gui_event);
                    consumed
                } 
                WindowEvent::CursorMoved { position, .. } => {
                    let pos = apply_scale_factor(*position, self.scale_factor);
    
                    let consumed = self.gui.mouse_moved(pos.x as u32, pos.y as u32);
                    consumed
                },
                WindowEvent::Touch(touch) => {
                    let pos = apply_scale_factor(touch.location, self.scale_factor);
    
                    match touch.phase {
                        TouchPhase::Started => {
                            let _consumed = self.gui.mouse_moved(pos.x as u32, pos.y as u32);
                            let (_consumed, gui_event) = self.gui.mouse_pressed(true);
                            self.handle_gui_event(gui_event);
                        }
                        TouchPhase::Ended => {
                            let (_consumed, gui_event) = self.gui.mouse_pressed(false);
                            self.handle_gui_event(gui_event);
                        }
                        TouchPhase::Cancelled => {
                            let (_consumed, gui_event) = self.gui.mouse_pressed(false);
                            self.handle_gui_event(gui_event);
                        }
                        TouchPhase::Moved => {
                            let _consumed = self.gui.mouse_moved(pos.x as u32, pos.y as u32);
                        }
                    }
                    true
                } 
                _ => false,
            };
        self.performance_monitor.watch.stop(2);

        res
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render(
            &[&self.textured_quad],
            &self.gui,
            &mut self.performance_monitor)
    }


}



#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub async fn run()
{
    let default_window = default_window::DefaultWindow::new();
    let app = SchwarzschildRaytracer::new(&default_window.window).await;

    default_window::run(default_window, app);
}