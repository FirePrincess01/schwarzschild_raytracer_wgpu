//! resource loading functionalities from "Learn Wgpu" tutorial

use std::io::{BufReader, Cursor};

use cfg_if::cfg_if;
use glam::Vec3;
use wgpu::util::DeviceExt;
use crate::schwarzschild_object_shader::model;

// For wasm you need to manually copy the resources folder
#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let mut origin = location.origin().unwrap();
    if !origin.ends_with("schwarzschild_raytracer/resources") {
        origin = format!("{}/schwarzschild_raytracer/resources", origin);
    }
    let base = reqwest::Url::parse(&format!("{}/", origin,)).unwrap();
    base.join(file_name).unwrap()
}

// Loads a text file from the resources folder
pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("resources")
                .join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

// Loads a binary file from the resources folder
pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("resources")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}

// Creates a texture from a file name
pub async fn load_texture(
    file_name: &str,
    wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface,
    texture_bind_group_layout: &wgpu_renderer::vertex_texture_shader::TextureBindGroupLayout,
    nr_mipmaps: u32,
) -> anyhow::Result<wgpu_renderer::vertex_texture_shader::Texture> {
    let data = load_binary(file_name).await?;
    wgpu_renderer::vertex_texture_shader::Texture::new_from_bytes(wgpu_renderer, texture_bind_group_layout, &data, file_name, nr_mipmaps)
}

// Loads a .obj model with texture and materials
// save_positions is an option to save vertex positions in CPU-memory
pub async fn load_model(
    file_name: &str,
    wgpu_renderer: &mut impl wgpu_renderer::renderer::WgpuRendererInterface,
    texture_bind_group_layout: &wgpu_renderer::vertex_texture_shader::TextureBindGroupLayout,
    save_positions: bool,
) -> anyhow::Result<model::Model> {
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();
    for m in obj_materials? {
        let diffuse_texture = load_texture(&m.diffuse_texture.unwrap(), wgpu_renderer, texture_bind_group_layout, 4).await?;

        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
        })
    }

    let meshes = models.iter().map(|m| {
        let vertices = (0..m.mesh.positions.len() / 3)
            .map(|i| crate::schwarzschild_object_shader::vertex::Vertex {
                position: [
                    m.mesh.positions[i * 3],
                    m.mesh.positions[i * 3 + 1],
                    m.mesh.positions[i * 3 + 2],
                ],
                tex_coords: [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]],
                normal: [
                    m.mesh.normals[i * 3],
                    m.mesh.normals[i * 3 + 1],
                    m.mesh.normals[i * 3 + 2],
                ],
            })
            .collect::<Vec<_>>();

        let vertex_buffer = wgpu_renderer.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Vertex Buffer", file_name)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = wgpu_renderer.device().create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{:?} Index Buffer", file_name)),
            contents: bytemuck::cast_slice(&m.mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        model::Mesh {
            name: file_name.to_string(),
            vertex_buffer,
            index_buffer,
            num_elements: m.mesh.indices.len() as u32,
            material: m.mesh.material_id.unwrap_or(0),
        }
    })
    .collect::<Vec<_>>();

    let positions = if save_positions { 
        models.into_iter().map(|m| {
            (0..m.mesh.positions.len() / 3)
                .map(|i| Vec3::new( 
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                ))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut sum = 0;
    for i in 0..positions.len() {
        sum += positions[i].len();
    }

    log::info!("Loaded Model with {} vertices", sum);

    Ok(model::Model { meshes, materials, positions })
}