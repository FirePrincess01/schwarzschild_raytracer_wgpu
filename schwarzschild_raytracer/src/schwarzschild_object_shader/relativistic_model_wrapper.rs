//! This class contains a regular model and adds computation and buffers for the light rays

use glam::Vec3;
use crate::simulation::ray_connector::RayConnector;
use wgpu::{core::instance, util::DeviceExt};

use super::{model_matrix_bind_group_layout::ModelMatrixBindGroupLayout, model_matrix_uniform_buffers::ModelMatrixUniformBuffer, schwarzschild_object_shader_draw::SchwarzschildObjectShaderDraw};

pub struct RelativisticModelWrapper {
    model: super::model::Model,

    pub model_matrix: glam::Mat4,
    matrix_buffer: ModelMatrixUniformBuffer,

    has_farside: bool,
    points: Vec<Vec<RayConnector>>,
    points_farside: Vec<Vec<RayConnector>>,
    light_buffers: Vec<wgpu::Buffer>, 
    light_buffers_farside: Vec<wgpu::Buffer>,

    start_time: instant::Instant,
}

impl RelativisticModelWrapper {
    
    
    
    
    pub fn new(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        model: super::model::Model, model_matrix: glam::Mat4, 
        matrix_bind_group_layout: &ModelMatrixBindGroupLayout,
        schwarz_r: f32, observer_pos: Vec3, has_farside: bool) -> Self {
            
        let matrix_buffer = ModelMatrixUniformBuffer::new(wgpu_renderer.device(), matrix_bind_group_layout);

        let points = model.positions.iter().map(|mesh|{
            mesh.iter().map(|pos| {
                let mut ray = RayConnector::new(schwarz_r, model_matrix.transform_point3(*pos), true);
                ray.reset_ray(observer_pos);
                ray
            })
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

        let points_farside: Vec<Vec<RayConnector>>;
        if has_farside {
            points_farside = model.positions.iter().map(|mesh|{
                mesh.iter().map(|pos| {
                    let mut ray = RayConnector::new(schwarz_r, model_matrix.transform_point3(*pos), false);
                    ray.reset_ray(observer_pos);
                    ray
                })
                .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        } else {
            points_farside = Vec::new();
        }

        let light_buffers = points.iter().map(|mesh| {
            let light_angles = mesh.iter().map(|point| {
                point.last_angle()
            })
            .collect::<Vec<_>>();
            
            wgpu_renderer.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Light Buffer", mesh.len())),
                contents: bytemuck::cast_slice(&light_angles),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            })
        })
        .collect::<Vec<_>>();

        let mut light_buffers_farside = Vec::new();
        if has_farside {
            light_buffers_farside = points_farside.iter().map(|mesh| {
                let light_angles = mesh.iter().map(|point| {
                    point.last_angle()
                })
                .collect::<Vec<_>>();
                
                wgpu_renderer.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{:?} Light Buffer farside", mesh.len())),
                    contents: bytemuck::cast_slice(&light_angles),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                })
            })
            .collect::<Vec<_>>();
        }
        
        Self {
            model,
            model_matrix,
            matrix_buffer,
            light_buffers,
            light_buffers_farside,
            has_farside,
            points,
            points_farside,
            start_time: instant::Instant::now(),
        }
    }

    #[allow(dead_code)]
    pub fn has_farside(&self) -> bool {
        self.has_farside
    }

    pub fn update(&mut self, queue: &wgpu::Queue, observer_pos: Vec3) {
        //self.model_matrix = self.model_matrix * glam::Mat4::from_rotation_z(0.002);
        let seconds = (instant::Instant::now() - self.start_time).as_secs_f32();
        self.model_matrix = glam::Mat4::from_rotation_z(0.2 * seconds) *
            glam::Mat4::from_translation(Vec3::new(20., 0., 0.)) *
            glam::Mat4::from_rotation_z(1. * seconds) * 
            glam::Mat4::from_scale(Vec3::new(3., 3., 3.));
        self.matrix_buffer.update(queue, &self.model_matrix);

        for i in 0..self.points.len() {
            for j in 0..self.points[i].len() {
                let pos = self.model_matrix.transform_point3(self.model.positions[i][j]);
                
                self.points[i][j].set_position(pos);
                if self.has_farside
                {
                    self.points_farside[i][j].set_position(pos);
                }
            }

        }
        
        for i in 0..self.points.len() {
            self.points[i].iter_mut().for_each(|point| { 
                point.update_ray(observer_pos, 1); 
            });
            let angles = self.points[i].iter().map(|point| {
                    point.last_angle()
                })
                .collect::<Vec<_>>();
            let data = bytemuck::cast_slice( &angles );

            if self.light_buffers[i].size() == data.len() as u64 {
                queue.write_buffer(&self.light_buffers[i], 0, data);
            } else {
                log::error!("WTF light buffer wrong size");
            }

            if self.has_farside {
                self.points_farside[i].iter_mut().for_each(|point| { 
                    point.update_ray(observer_pos, 1); 
                });
                let angles = self.points_farside[i].iter().map(|point| {
                        point.last_angle()
                    })
                    .collect::<Vec<_>>();
                let data = bytemuck::cast_slice( &angles );
    
                if self.light_buffers_farside[i].size() == data.len() as u64 {
                    queue.write_buffer(&self.light_buffers_farside[i], 0, data);
                } else {
                    log::error!("WTF light farside buffer wrong size");
                }
            }
        }
    }
}

impl SchwarzschildObjectShaderDraw for RelativisticModelWrapper {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.matrix_buffer.bind(render_pass);

        for i in 0..self.points.len() {
            let num_elements = self.model.meshes[i].num_elements;
            self.model.meshes[i].bind(render_pass);
            self.model.materials[self.model.meshes[i].material].diffuse_texture.bind(render_pass);

            render_pass.set_vertex_buffer(1, self.light_buffers[i].slice(..));
            render_pass.draw_indexed(0..num_elements, 0, 0..1);
            if self.has_farside {
                render_pass.set_vertex_buffer(1, self.light_buffers_farside[i].slice(..));
                render_pass.draw_indexed(0..num_elements, 0, 0..1);
            }
        }
    }
}