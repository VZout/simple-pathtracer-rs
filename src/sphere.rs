use crate::shape::*;
use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::{BHShape};

pub struct Sphere
{
    pub pos: glm::Vec3,
    pub radius: f32,
    pub material_id: u32,
    pub node_index: usize,
}

impl Bounded for Sphere
{
    fn aabb(&self) -> AABB
    {

        let half_size = bvh::nalgebra::Vector3::new(self.radius, self.radius, self.radius);
        let nalgebra_pos = bvh::nalgebra::Point3::new(self.pos.x, self.pos.y, self.pos.z);
        let min = nalgebra_pos - half_size;
        let max = nalgebra_pos + half_size;
        AABB::with_bounds(min, max)
    }
}

impl BHShape for Sphere
{
    fn set_bh_node_index(&mut self, index: usize)
    {
        self.node_index = index;
    }

    fn bh_node_index(&self) -> usize
    {
        self.node_index
    }
}

impl Shape for Sphere {
    fn intersect(&self, origin: glm::Vec3, direction: glm::Vec3, _barry: &mut glm::Vec2) -> Option<f32>
    {
        let radius2 = self.radius;

        let l = self.pos - origin;
        let tca = glm::dot(l, direction);

        if tca < 0f32
        {
            return None;
        }

        let d2 = glm::dot(l, l) - tca * tca;

        if d2 > radius2
        {
            return None;
        }

        let thc = (radius2 - d2).sqrt();
        let mut t0 = tca - thc;
        let mut t1 = tca + thc;

        if t0 > t1
        {
            std::mem::swap(&mut t0, &mut t1);
        }

        if t0 < 0f32
        {
            t0 = t1; // if t0 is negative, let's use t1 instead
            if t0 < 0f32
            {
                return None; // both t0 and t1 are negative
            }
        }

        return Some(t0);
    }

    fn get_normal(&self, hit_pos: glm::Vec3, _barry: glm::Vec2) -> glm::Vec3
    {
        return glm::normalize(hit_pos - self.pos);
    }

    fn get_material_id(&self) -> u32
    {
        return self.material_id;
    }

    fn get_tangents(&self, _normal: glm::Vec3, _tangent: &mut glm::Vec3, _bitangent: &mut glm::Vec3, _barry: glm::Vec2) {}
}