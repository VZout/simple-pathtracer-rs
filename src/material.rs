use crate::resource_manager::*;

pub type MaterialManager = ResourceManager<Material, MaterialLoader>;

#[derive(Copy, Clone)]
pub struct Material
{
    pub color: glm::Vec3,
    pub metallic: f32,
    pub specular: f32,
    pub roughness: f32,
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
            roughness: 0.5f32
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

// Some pre-made materials.
pub mod materials {
    use crate::material::Material;

    pub static GLOSSY_WHITE:   Material = Material { color: glm::Vec3 { x: 1f32, y: 1f32, z: 1f32 }, metallic: 0.9f32, specular: 0.5f32, roughness: 0.4f32, };
    pub static GREEN: Material = Material { color: glm::Vec3 { x: 0f32, y: 1f32, z: 0f32 }, metallic: 0f32, specular: 0.5f32, roughness: 1.0f32, };
    pub static BLUE:  Material = Material { color: glm::Vec3 { x: 0f32, y: 0f32, z: 1f32 }, metallic: 0f32, specular: 0.5f32, roughness: 1.0f32, };
    pub static GLOSSY_ORANGE:  Material = Material { color: glm::Vec3 { x: 0.8f32, y: 0.4f32, z: 0f32 }, metallic: 0.9f32, specular: 0.5f32, roughness: 0.1f32, };

}