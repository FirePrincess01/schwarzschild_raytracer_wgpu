//! Contains the render pipelines needed to render the frames
//! Controls the render process and simulation on a top level

use std::f64::consts::FRAC_PI_2;

use crate::performance_monitor::PerformanceMonitor;
use crate::schwarzschild_sphere_shader::schwarzschild_sphere_shader_draw::SchwarzschildSphereShaderDraw;
use crate::schwarzschild_sphere_shader::sphere_observer_bind_group_layout::SphereObserverBindGroupLayout;
use crate::schwarzschild_sphere_shader::sphere_observer_uniform_buffers::SphereObserverUniformBuffer;
use crate::schwarzschild_sphere_shader::ray_fan_bind_group_layout;
use crate::simulation::observer::Observer;
use crate::{schwarzschild_sphere_shader, simulation};
use glam::DVec2;
use wgpu_renderer::renderer::WgpuRenderer;
use wgpu_renderer::vertex_color_shader::{self, VertexColorShaderDraw};
use wgpu_renderer::vertex_texture_shader::{self, VertexTextureShaderDraw};
use winit::event::{VirtualKeyCode, ElementState, MouseScrollDelta};

use super::observer_controller::ObserverController;

pub struct Renderer 
{   
    // wgpu_renderer
    pub wgpu_renderer: WgpuRenderer,

    pub observer: simulation::observer::Observer,

    pipeline_sphere: schwarzschild_sphere_shader::pipeline::Pipeline,
    pub ray_fan_bind_group_layout: ray_fan_bind_group_layout::RayFanBindGroupLayout,
    pub sphere_observer_bind_group_layout: SphereObserverBindGroupLayout,
    pub sphere_observer_uniform_buffer: SphereObserverUniformBuffer,
    pub texture_bind_group_layout: vertex_texture_shader::TextureBindGroupLayout,

    pipeline_lines: vertex_color_shader::Pipeline,
    pipeline_texture_gui: vertex_texture_shader::Pipeline,
    pub camera_bind_group_layout: vertex_color_shader::CameraBindGroupLayout,
    camera_uniform_orthographic: vertex_color_shader::CameraUniform,
    camera_uniform_orthographic_buffer: vertex_color_shader::CameraUniformBuffer,

    camera_controller: ObserverController,

    mouse_pressed: bool,
    last_mouse_position: DVec2,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> Self 
    {   
        // wgpu renderer
        let mut wgpu_renderer = WgpuRenderer::new(window).await; 
        let surface_format = wgpu_renderer.config().format;
        let surface_width = wgpu_renderer.config().width;
        let surface_height = wgpu_renderer.config().height;
        //let surface_format = wgpu_renderer.config().format;
        
        let ray_fan_bind_group_layout = ray_fan_bind_group_layout::RayFanBindGroupLayout::new(wgpu_renderer.device());
        let sphere_observer_bind_group_layout = SphereObserverBindGroupLayout::new(wgpu_renderer.device());
        let texture_bind_group_layout = vertex_texture_shader::TextureBindGroupLayout::new(wgpu_renderer.device());
        let pipeline_sphere = schwarzschild_sphere_shader::pipeline::Pipeline::new(
            wgpu_renderer.device(),
            &sphere_observer_bind_group_layout,
            &ray_fan_bind_group_layout,
            &texture_bind_group_layout,
            surface_format,
            true,
        );

        let sphere_observer_uniform_buffer = SphereObserverUniformBuffer::new(wgpu_renderer.device(), &sphere_observer_bind_group_layout);

        let camera_bind_group_layout = vertex_texture_shader::CameraBindGroupLayout::new(wgpu_renderer.device());
        // pipeline lines
        let pipeline_lines = vertex_color_shader::Pipeline::new_lines(
            wgpu_renderer.device(), 
            &camera_bind_group_layout, 
            surface_format,
        );

        let schwarz_r = 10.;
        let fov = FRAC_PI_2;
        let observer = Observer::new(schwarz_r, fov, surface_width as f64, surface_height as f64);

        // pipeline texture gui
        let texture_bind_group_layout = vertex_texture_shader::TextureBindGroupLayout::new(wgpu_renderer.device());
        let pipeline_texture_gui = vertex_texture_shader::Pipeline::new_gui(
            wgpu_renderer.device(), 
            &camera_bind_group_layout, 
            &texture_bind_group_layout, 
            surface_format
        );


        // processes user inputs regarding movement and camera
        let speed = 8.0;
        let sensitivity = 1.0;
        let sensitivity_scroll = 1.0;
        let camera_controller = super::observer_controller::ObserverController::new(speed, sensitivity, sensitivity_scroll);


        let width = wgpu_renderer.config().width;
        let height = wgpu_renderer.config().height;
        let camera_uniform_orthographic: vertex_color_shader::CameraUniform = vertex_color_shader::CameraUniform::new_orthographic(width, height);
        let mut camera_uniform_orthographic_buffer = vertex_color_shader::CameraUniformBuffer::new(
                wgpu_renderer.device(), 
                &camera_bind_group_layout);

        camera_uniform_orthographic_buffer.update(wgpu_renderer.queue(), camera_uniform_orthographic);   // add uniform identity matrix

        Self {
            wgpu_renderer,
            observer,
            pipeline_sphere,
            ray_fan_bind_group_layout,
            sphere_observer_bind_group_layout,
            sphere_observer_uniform_buffer,
            texture_bind_group_layout,
            pipeline_lines,
            camera_bind_group_layout,

            pipeline_texture_gui,
            camera_uniform_orthographic,
            camera_uniform_orthographic_buffer,
            camera_controller,
            mouse_pressed: false,
            last_mouse_position: DVec2::ZERO,
        } 
    }




    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // self.size = new_size;
        self.observer.update_screen_format(new_size.width as f64, new_size.height as f64);

        //self.camera_projection.resize(new_size.width, new_size.height);
        self.wgpu_renderer.resize(new_size);
    
        self.camera_uniform_orthographic.resize_orthographic(new_size.width, new_size.height);
        self.camera_uniform_orthographic_buffer.update(self.wgpu_renderer.queue(), self.camera_uniform_orthographic);
    }

    pub fn update(&mut self, dt: instant::Duration) {
        self.camera_controller.update_observer(&mut self.observer, dt);
        let observer_pipeline = self.observer.calc_transformation_pipeline();
        self.sphere_observer_uniform_buffer.update(self.wgpu_renderer.queue(), observer_pipeline);

        // camera
        // self.camera_controller.update_camera(&mut self.camera, dt);
        // self.camera_uniform.update_view_proj(&self.camera, &self.camera_projection);
        // self.camera_uniform_buffer.update(self.wgpu_renderer.queue(), self.camera_uniform);
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool 
    {
        let res = match key {
            VirtualKeyCode::Key1 => {
                self.observer.start_unmoving();
                true
            },
            VirtualKeyCode::Key2 => {
                self.observer.start_frozen_fall();
                true
            },
            VirtualKeyCode::Key3 => {
                self.observer.start_orbit(0.);
                true
            },
            VirtualKeyCode::Key4 => {
                self.observer.start_orbit(18.);
                true
            },
            _ => false,
        };
        if res {return true;}
        self.camera_controller.process_keyboard(key, state)
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) 
    {
        // wont be using that now
        self.camera_controller.process_scroll(delta);
    }

    pub fn process_mouse_position(&mut self, x: f64, y: f64) {
        if self.mouse_pressed {
            let delta_x = x - self.last_mouse_position.x;
            let delta_y = y - self.last_mouse_position.y;
            self.camera_controller.process_mouse(delta_x, delta_y);
        }
        self.last_mouse_position.x = x;
        self.last_mouse_position.y = y;
    }

    pub fn set_mouse_pressed(&mut self, pressed: bool) {
        self.mouse_pressed = pressed;
    }

    pub fn render(&mut self, 
        spheres: &[&dyn SchwarzschildSphereShaderDraw],
        mesh_gui: & impl VertexTextureShaderDraw,
        performance_monitor: &mut PerformanceMonitor) -> Result<(), wgpu::SurfaceError>
    {
        performance_monitor.watch.start(0);
        let output = self.wgpu_renderer.get_current_texture()?;
        performance_monitor.watch.stop(0);

        performance_monitor.watch.start(1);

        let view: wgpu::TextureView = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder: wgpu::CommandEncoder = self.wgpu_renderer.device().create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { 
                label: Some("Forward Render Pass"), 
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.,
                            g: 0.,
                            b: 0.,
                            a: 1.0,
                        }),
                        store: true,
                    }
                })], 
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.wgpu_renderer.get_depth_texture_view(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }) 
            });

            self.pipeline_sphere.bind(&mut render_pass);
            self.sphere_observer_uniform_buffer.bind(&mut render_pass);
            for sphere in spheres {
                sphere.draw(&mut render_pass);
            }

            // performance monitor
            self.pipeline_lines.bind(&mut render_pass);
            self.camera_uniform_orthographic_buffer.bind(&mut render_pass);
            performance_monitor.draw(&mut render_pass);

            // gui
            self.pipeline_texture_gui.bind(&mut render_pass);
            self.camera_uniform_orthographic_buffer.bind(&mut render_pass);
            mesh_gui.draw(&mut render_pass);
        }

        self.wgpu_renderer.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        performance_monitor.watch.stop(1);
        
        Ok(())
    }

    pub fn get_schwarz_r(&self) -> f64 {
        return self.observer.get_schwarz_r();
    }

    pub fn get_radial_position(&self) -> f64 {
        return self.observer.get_radial_position();
    }
}


