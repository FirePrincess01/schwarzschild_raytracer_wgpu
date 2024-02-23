//! This class contains a normal model and adds computation and buffers for the light rays

use glam::Vec3;
use crate::simulation::ray_connector::RayConnector;
use wgpu::util::DeviceExt;

use super::schwarzschild_object_shader_draw::SchwarzschildObjectShaderDraw;

pub struct RelativisticModelWrapper {
    model: super::model::Model,
    pub model_matrix: glam::Mat4,
    //uniform buffer
    light_buffers: Vec<wgpu::Buffer>, 
    light_buffers_farside: Vec<wgpu::Buffer>,
    has_farside: bool,
    points: Vec<Vec<RayConnector>>,
    points_farside: Vec<Vec<RayConnector>>,
}

impl RelativisticModelWrapper {
    
    
    
    
    pub fn new(wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface, 
        model: super::model::Model, model_matrix: glam::Mat4, 
        schwarz_r: f32, observer_pos: Vec3, has_farside: bool) -> Self {
        let points = model.positions.iter().map(|mesh|{
            mesh.iter().map(|pos| {
                let mut ray = RayConnector::new(schwarz_r, *pos, true);
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
                    let mut ray = RayConnector::new(schwarz_r, *pos, false);
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
            light_buffers,
            light_buffers_farside,
            has_farside,
            points,
            points_farside,
        }
    }

    #[allow(dead_code)]
    pub fn has_farside(&self) -> bool {
        self.has_farside
    }

    pub fn update(&mut self, queue: &wgpu::Queue, observer_pos: Vec3) {
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

        //maybe move the object?

        //update the model matrix too!
    }
}

impl SchwarzschildObjectShaderDraw for RelativisticModelWrapper {
    fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for i in 0..self.points.len() {
            let num_elements = self.model.meshes[i].num_elements;
            self.model.meshes[i].bind(render_pass);
            //self.model.materials[self.model.meshes[i].material].diffuse_texture.bind(render_pass);

            render_pass.set_vertex_buffer(1, self.light_buffers[i].slice(..));
            render_pass.draw_indexed(0..num_elements, 0, 0..1);
            if self.has_farside {
                render_pass.set_vertex_buffer(1, self.light_buffers_farside[i].slice(..));
                render_pass.draw_indexed(0..num_elements, 0, 0..1);
            }
        }
    }
}