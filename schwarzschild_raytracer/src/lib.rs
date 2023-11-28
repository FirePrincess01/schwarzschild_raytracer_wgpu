//! The main class of this programm, dealing with all the interactions with webgpu
//! Basically the main file of this program

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
    fps: wgpu_renderer::performance_monitor::Fps,

    // data
    first_sphere: BasicSphereBuffer,
    second_sphere: BasicSphereBuffer,
    third_sphere: BasicSphereBuffer,

    // gui
    font: rusttype::Font<'static>,
    gui: gui::Gui,

    rotation_selection_mode: bool,
    selected_rotation: f64,
    rotation_delta: f64,
}

impl SchwarzschildRaytracer {
    pub async fn new(window: &winit::window::Window) -> Self 
    {
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
        let width = size.width;
        let height = size.height;

        let mut renderer = renderer::Renderer::new(window).await;
        let mut performance_monitor = performance_monitor::PerformanceMonitor::new(
            &mut renderer.wgpu_renderer);
        performance_monitor.show = false;

        let fps = wgpu_renderer::performance_monitor::Fps::new();

        let texture_image = image::load_from_memory(include_bytes!("eso0932a.jpg")).unwrap();
        let texture_image2 = image::load_from_memory(include_bytes!("world_8k.png")).unwrap();
        let texture_image3 = image::load_from_memory(include_bytes!("transparent_clouds.png")).unwrap();

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

        let third_sphere = BasicSphereBuffer::new(
            &mut renderer.wgpu_renderer, 
            &renderer.texture_bind_group_layout, 
            &renderer.ray_fan_bind_group_layout, 
            12., 
            schwarz_r, 
            &texture_image3);

        //Gui
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
            fps,

            first_sphere,
            second_sphere,
            third_sphere,

            font,
            gui,

            rotation_selection_mode: false,
            selected_rotation: 18.,
            rotation_delta: 0.,
        }
    }

    fn handle_gui_event(&mut self, 
        gui_result: &gui::GuiResult)
    {
        match &gui_result.pressed_event {
            Some(event) => {
                match event {
                    gui::PressedEvent::MovementButton(id) => {
                        match id {
                            gui::MovementButtonId::Up => {
                                self.renderer.process_keyboard(VirtualKeyCode::Space, ElementState::Pressed);
                            },
                            gui::MovementButtonId::Forward => {
                                if self.rotation_selection_mode {
                                    self.rotation_delta += 1.;
                                }
                                else {
                                    self.renderer.process_keyboard(VirtualKeyCode::W, ElementState::Pressed);
                                }
                            },
                            gui::MovementButtonId::Down => {
                                self.renderer.process_keyboard(VirtualKeyCode::LShift, ElementState::Pressed);
                            },  
                            gui::MovementButtonId::Left => {
                                self.renderer.process_keyboard(VirtualKeyCode::A, ElementState::Pressed);
                            },
                            gui::MovementButtonId::Back => {
                                if self.rotation_selection_mode {
                                    self.rotation_delta -= 1.;
                                }
                                else {
                                    self.renderer.process_keyboard(VirtualKeyCode::S, ElementState::Pressed);
                                }
                            },
                            gui::MovementButtonId::Right => {
                                self.renderer.process_keyboard(VirtualKeyCode::D, ElementState::Pressed);
                            },
                        }
                    },
                }
            },
            None => {},
        }

        match &gui_result.released_event {
            Some(event) => {
                match event {
                    gui::ReleasedEvent::SideButton(id) => {
                        match id {
                            gui::SideButtonId::Reset => { self.renderer.observer.reset_to_start(); },
                            gui::SideButtonId::Still => { self.renderer.observer.start_unmoving(); },
                            gui::SideButtonId::FrozenFall => { self.renderer.observer.start_frozen_fall(); },
                            gui::SideButtonId::Fall => { self.renderer.observer.start_orbit(0.); },
                            gui::SideButtonId::Orbit => { 
                                self.selected_rotation = 18.;
                                self.rotation_delta = 0.;
                                self.rotation_selection_mode = true
                            },
                            gui::SideButtonId::PerformanceMonitor => { self.performance_monitor.show = !self.performance_monitor.show; },
                        }
                    },
                    gui::ReleasedEvent::MovementButton(id) => {
                        match id {
                            gui::MovementButtonId::Up => {
                                self.renderer.process_keyboard(VirtualKeyCode::Space, ElementState::Released);
                            },
                            gui::MovementButtonId::Forward => {
                                if self.rotation_selection_mode {
                                    self.rotation_delta += -1.;
                                }
                                else {
                                    self.renderer.process_keyboard(VirtualKeyCode::W, ElementState::Released);
                                }
                            },
                            gui::MovementButtonId::Down => {
                                self.renderer.process_keyboard(VirtualKeyCode::LShift, ElementState::Released);
                            },  
                            gui::MovementButtonId::Left => {
                                self.renderer.process_keyboard(VirtualKeyCode::A, ElementState::Released);
                            },
                            gui::MovementButtonId::Back => {
                                if self.rotation_selection_mode {
                                    self.rotation_delta -= -1.;
                                }
                                else {
                                    self.renderer.process_keyboard(VirtualKeyCode::S, ElementState::Released);
                                }
                            },
                            gui::MovementButtonId::Right => {
                                self.renderer.process_keyboard(VirtualKeyCode::D, ElementState::Released);
                            },
                        }
                    },
                    gui::ReleasedEvent::AdjustSpin(id) => {
                        match id {
                            gui::AdjustSpinButtonId::Confirm => {
                                self.rotation_selection_mode = false;
                                self.renderer.observer.start_orbit(self.selected_rotation);
                            },
                        }
                    },
                }
            },
            None => {},
        }
    }

    fn update_rotation_gui(&mut self, dt: instant::Duration) {
        //if self.rotation_delta.abs() < 0.00001 {
        //    return;
        //}
        self.selected_rotation += self.rotation_delta * dt.as_secs_f64();
        self.gui.adjust_spin_set_value(&mut self.renderer.wgpu_renderer, &self.font, self.selected_rotation as u32);
        let stability = simulation::orbit::Orbit::is_stable(self.selected_rotation, self.renderer.get_schwarz_r(), self.renderer.get_radial_position());
        match stability {
            simulation::orbit::OrbitStability::HittingSingularity => {self.gui.adjust_spin_set_colors(true, false, false);},
            simulation::orbit::OrbitStability::StableOrbit => {self.gui.adjust_spin_set_colors(false, true, false);},
            simulation::orbit::OrbitStability::EscapeTrajectory => {self.gui.adjust_spin_set_colors(false, false, true);},
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
        self.update_rotation_gui(dt);
        self.renderer.update(dt);

        self.performance_monitor.watch.start(3);
            let r = self.renderer.get_radial_position();
            self.first_sphere.update_ray_fan(self.renderer.wgpu_renderer.queue(), r);
            self.second_sphere.update_ray_fan(self.renderer.wgpu_renderer.queue(), r);
            self.third_sphere.update_ray_fan(self.renderer.wgpu_renderer.queue(), r);
        self.performance_monitor.watch.stop(3);
        
        // self.performance_monitor.watch.start(4);
        //     // update more stuff
        // self.performance_monitor.watch.stop(4);

        self.performance_monitor.update(&mut self.renderer.wgpu_renderer);

        // gui fps
        self.fps.update(dt);
        self.gui.fps_counter_set_value(&mut self.renderer.wgpu_renderer, &self.font, self.fps.get());
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
                    let res = self.gui.mouse_moved(pos.x as u32, pos.y as u32);
                    self.handle_gui_event(&res);

                    if !res.consumed {
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
                    
                    let res = self.gui.mouse_pressed(is_pressed);
                    self.handle_gui_event(&res);

                    if !res.consumed {
                        self.renderer.set_mouse_pressed(*state == ElementState::Pressed);
                    }
                    true
                }
                WindowEvent::Touch(touch) => {
                    let pos = apply_scale_factor(touch.location, self.scale_factor);
    
                    match touch.phase {
                        TouchPhase::Started => {
                            let res = self.gui.mouse_moved(pos.x as u32, pos.y as u32);
                            self.handle_gui_event(&res);
                            let res = self.gui.mouse_pressed(true);
                            self.handle_gui_event(&res);
    
                            if !res.consumed {
                                self.renderer.process_mouse_position(pos.x as f64, pos.y as f64);
                                self.renderer.set_mouse_pressed(true);
                            }
                        }
                        TouchPhase::Ended => {
                            let res = self.gui.mouse_pressed(false);
                            self.handle_gui_event(&res);
    
                            self.renderer.set_mouse_pressed(false);
                        }
                        TouchPhase::Cancelled => {
                            let res = self.gui.mouse_pressed(false);
                            self.handle_gui_event(&res);
                            
                            self.renderer.set_mouse_pressed(false);
                        }
                        TouchPhase::Moved => {
                            let res = self.gui.mouse_moved(pos.x as u32, pos.y as u32);
                            self.handle_gui_event(&res);

                            if !res.consumed {
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
            &[&self.first_sphere, &self.second_sphere, &self.third_sphere],
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