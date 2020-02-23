extern crate glm;

pub trait Shape
{
    fn intersect(&self, origin: glm::Vec3, direction: glm::Vec3) -> Option<f32>;
    fn get_normal(&self, hit_pos: glm::Vec3) -> glm::Vec3;
    fn get_material_id(&self) -> u32;
}