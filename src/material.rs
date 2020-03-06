extern crate image;

use crate::resource_manager::*;
use crate::texture::TextureManager;
use crate::scene::*;
use crate::disney;
use std::collections::HashMap;
use std::borrow::BorrowMut;
use image::{GenericImageView, DynamicImage};

pub type MaterialManager = ResourceManager<Material, MaterialLoader>;

#[derive(Copy, Clone)]
pub struct Material
{
    pub color: glm::Vec3,
    pub metallic: f32,
    pub specular: f32,
    pub roughness: f32,
    pub albedo_id: Option<u32>,
    pub roughness_id: Option<u32>,
    pub metallic_id: Option<u32>,
}

pub struct SurfaceMaterial
{
    pub color: glm::Vec3,
    pub metallic: f32,
    pub specular: f32,
    pub roughness: f32,
    pub cs_w: f32,
}

impl Default for Material
{
    fn default() -> Self
    {
        Material
        {
            color: glm::vec3(0f32, 0f32, 0f32),
            metallic: 0f32,
            specular: 0.5f32,
            roughness: 0.5f32,
            albedo_id: None,
            roughness_id: None,
            metallic_id: None,
        }
    }
}

pub struct MaterialLoader
{
}

impl ResourceLoader<Material> for MaterialLoader {
    type Args = Material;
    fn load(&self, details: &Material) -> Result<Material, String> {
        Ok(details.clone())
    }
}

pub fn load_textures_of_material(material_id: &u32, material_manager: &mut MaterialManager, texture_manager: &mut TextureManager)
{
    let mut material = *material_manager.get(material_id).unwrap();

    material.albedo_id = Some(texture_manager.load("iron_mat/albedo.png"));
    material.roughness_id = Some(texture_manager.load("iron_mat/roughness.png"));
    material.metallic_id = Some(texture_manager.load("iron_mat/metallic.png"));

    material_manager.set(*material_id, material);
}

pub fn get_texture_xy(hit: &Hit, image: &DynamicImage) -> glm::UVec2
{
    let scale = 5f32;
    let x = (image.width() as f32 * (hit.uv.x * scale)) as u32 % image.width();
    let y = (image.height() as f32 * (hit.uv.y * scale)) as u32 % image.height();

    return glm::uvec2(x, y);
}

pub fn sample_texture(image: &DynamicImage, xy: glm::UVec2, gamma_correct: bool) -> glm::Vec3
{
    let pixel = image.get_pixel(xy.x, xy.y);

    if gamma_correct
    {
        return glm::vec3((pixel[0] as f32 / 255f32).powf(2.2),
                         (pixel[1] as f32 / 255f32).powf(2.2),
                         (pixel[2] as f32 / 255f32).powf(2.2));
    }

    return glm::vec3((pixel[0] as f32 / 255f32),
                       (pixel[1] as f32 / 255f32),
                       (pixel[2] as f32 / 255f32));
}

pub fn sample_texture_1d(image: &DynamicImage, xy: glm::UVec2, gamma_correct: bool) -> f32
{
    let pixel = image.get_pixel(xy.x, xy.y);
    return pixel[0] as f32 / 255f32;
}

pub fn get_surface_material(hit: &Hit, material_manager: &MaterialManager, texture_manager: &TextureManager) -> SurfaceMaterial
{
    let material = material_manager.get(&hit.material_id).unwrap();
    let mut surface_material = SurfaceMaterial
    {
        color: material.color,
        metallic: material.metallic,
        specular: material.specular,
        roughness: material.roughness,
        cs_w: 0f32,
    };

    // Calculate CSW
    surface_material.cs_w = disney::calculate_csw(&surface_material);

    // Albedo
    if let Some(id) = material.albedo_id
    {
        let texture = texture_manager.get(&id).unwrap();
        let xy = get_texture_xy(&hit, &texture);
        surface_material.color = sample_texture(&texture, xy, true);
    }

    // Roughness
    if let Some(id) = material.roughness_id
    {
        let texture = texture_manager.get(&id).unwrap();
        let xy = get_texture_xy(&hit, &texture);
        surface_material.roughness = sample_texture_1d(&texture, xy, false);
    }

    // Metallic
    if let Some(id) = material.metallic_id
    {
        let texture = texture_manager.get(&id).unwrap();
        let xy = get_texture_xy(&hit, &texture);
        surface_material.metallic = sample_texture_1d(&texture, xy, false);
    }

    return surface_material;
}

// Some pre-made materials.
pub mod materials {
    use crate::material::Material;

    pub static GLOSSY_WHITE:   Material = Material { color: glm::Vec3 { x: 1f32, y: 1f32, z: 1f32 }, metallic: 0.0f32, specular: 0.5f32, roughness: 0.4f32, albedo_id: None, roughness_id: None, metallic_id: None, };
    pub static GREEN: Material = Material { color: glm::Vec3 { x: 0f32, y: 1f32, z: 0f32 }, metallic: 0f32, specular: 0.5f32, roughness: 1.0f32, albedo_id: None, roughness_id: None, metallic_id: None, };
    pub static BLUE:  Material = Material { color: glm::Vec3 { x: 0f32, y: 0f32, z: 1f32 }, metallic: 0f32, specular: 0.5f32, roughness: 1.0f32, albedo_id: None, roughness_id: None, metallic_id: None, };
    pub static GLOSSY_ORANGE:  Material = Material { color: glm::Vec3 { x: 0.8f32, y: 0.4f32, z: 0f32 }, metallic: 0.9f32, specular: 0.5f32, roughness: 0.1f32, albedo_id: None, roughness_id: None, metallic_id: None, };

}