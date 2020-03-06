extern crate glm;

pub trait Shape: Sync + Send
{
    fn intersect(&self, origin: glm::Vec3, direction: glm::Vec3, barry: &mut glm::Vec2) -> Option<f32>;
    fn get_normal(&self, hit_pos: glm::Vec3, barry: glm::Vec2) -> glm::Vec3;
    fn get_material_id(&self) -> u32;
    fn get_tangents(&self, normal: glm::Vec3, tangent: &mut glm::Vec3, bitangent: &mut glm::Vec3, barry: glm::Vec2);
    fn get_uv(&self, barry: glm::Vec2) -> glm::Vec2;
}