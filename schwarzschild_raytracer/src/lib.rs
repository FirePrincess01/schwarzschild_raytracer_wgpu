
mod renderer;
mod geometry;
mod performance_monitor;
mod simulation;
mod schwarzschild_sphere_shader;
mod gui;

use schwarzschild_sphere_shader::sphere_buffer::basic_sphere_buffer::BasicSphereBuffer;
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
    //textured_quad: textured_quad::TexturedQuad,
    first_sphere: BasicSphereBuffer,
    second_sphere: BasicSphereBuffer,

    // gui
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

        let texture_image = image::load_from_memory(include_bytes!("eso0932a.jpg")).unwrap();
        let texture_image2 = image::load_from_memory(include_bytes!("world_8k.png")).unwrap();

        let schwarz_r = renderer.get_schwarz_r();
        let first_sphere = BasicSphereBuffer::new(
            &mut renderer.wgpu_renderer, 
            &renderer.texture_bind_group_layout, 
            &renderer.ray_fan_bind_group_layout, 
            500., 
            schwarz_r, 
            &texture_image);

        let second_sphere = BasicSphereBuffer::new(
            &mut renderer.wgpu_renderer, 
            &renderer.texture_bind_group_layout, 
            &renderer.ray_fan_bind_group_layout, 
            11., 
            schwarz_r, 
            &texture_image2);

        let gui = gui::Gui::new(&mut renderer.wgpu_renderer, 
            &renderer.texture_bind_group_layout, 
            width, 
            height);

        Self {
            size,
            scale_factor,

            renderer,
            performance_monitor,

            first_sphere,
            second_sphere,

            gui,
        }
    }

    fn handle_gui_event(&mut self, 
        gui_event: Option<wgpu_renderer::gui::RectanglePressedEvent<gui::SideButtonId>>)
    {
        match gui_event {
            Some(event) => {
                match event.rectangle_id {
                    gui::SideButtonId::Reset => { self.renderer.observer.reset_to_start(); },
                    gui::SideButtonId::Still => { self.renderer.observer.start_unmoving(); },
                    gui::SideButtonId::FrozenFall => { self.renderer.observer.start_frozen_fall(); },
                    gui::SideButtonId::Fall => { self.renderer.observer.start_orbit(0.); },
                    gui::SideButtonId::Orbit => { self.renderer.observer.start_orbit(18.); },
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

        self.performance_monitor.watch.start(3);
            let r = self.renderer.get_radial_position();
            self.first_sphere.update_ray_fan(self.renderer.wgpu_renderer.queue(), r);
            self.second_sphere.update_ray_fan(self.renderer.wgpu_renderer.queue(), r);
        self.performance_monitor.watch.stop(3);
        
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
                WindowEvent::CursorMoved { position, .. } => {
                    let pos = apply_scale_factor(*position, self.scale_factor);
                    let consumed = self.gui.mouse_moved(pos.x as u32, pos.y as u32);

                    if !consumed {
                        self.renderer.process_mouse_position(pos.x, pos.y);
                    }
                    true
                }
                WindowEvent::MouseInput { 
                    state, 
                    button: MouseButton::Left, 
                    .. 
                } => {
                    let is_pressed = *state == ElementState::Pressed;
                    
                    let (consumed, gui_event) = self.gui.mouse_pressed(is_pressed);
                    self.handle_gui_event(gui_event);

                    if !consumed {
                        self.renderer.set_mouse_pressed(*state == ElementState::Pressed);
                    }
                    true
                }
                WindowEvent::Touch(touch) => {
                    let pos = apply_scale_factor(touch.location, self.scale_factor);
    
                    match touch.phase {
                        TouchPhase::Started => {
                            let _consumed = self.gui.mouse_moved(pos.x as u32, pos.y as u32);
                            let (consumed, gui_event) = self.gui.mouse_pressed(true);
                            self.handle_gui_event(gui_event);
    
                            if !consumed {
                                self.renderer.process_mouse_position(pos.x as f64, pos.y as f64);
                                self.renderer.set_mouse_pressed(true);
                            }
                        }
                        TouchPhase::Ended => {
                            let (_consumed, gui_event) = self.gui.mouse_pressed(false);
                            self.handle_gui_event(gui_event);
    
                            self.renderer.set_mouse_pressed(false);
                        }
                        TouchPhase::Cancelled => {
                            let (_consumed, gui_event) = self.gui.mouse_pressed(false);
                            self.handle_gui_event(gui_event);
                            
                            self.renderer.set_mouse_pressed(false);
                        }
                        TouchPhase::Moved => {
                            let consumed = self.gui.mouse_moved(pos.x as u32, pos.y as u32);

                            if !consumed {
                                self.renderer.process_mouse_position(pos.x as f64, pos.y as f64);
                            }
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
            &[&self.first_sphere, &self.second_sphere],
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