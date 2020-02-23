use crate::resource_manager::*;

pub type MaterialManager = ResourceManager<Material>;

#[derive(Copy, Clone)]
pub struct Material
{
    pub color: glm::Vec3,
}

impl Default for Material
{
    fn default() -> Self
    {
        Material
        {
            color: glm::vec3(0f32, 0f32, 0f32),
        }
    }
}

// Some pre-made materials.
pub mod materials {
    use crate::material::Material;

    pub static RED:   Material = Material { color: glm::Vec3 { x: 1f32, y: 0f32, z: 0f32 }, };
    pub static GREEN: Material = Material { color: glm::Vec3 { x: 0f32, y: 1f32, z: 0f32 }, };
    pub static BLUE:  Material = Material { color: glm::Vec3 { x: 0f32, y: 0f32, z: 1f32 }, };

}