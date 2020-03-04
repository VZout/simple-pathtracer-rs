use crate::shape::*;
use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::{BHShape};
use crate::model::Vertex;

pub struct Triangle
{
    pub v0: Vertex,
    pub v1: Vertex,
    pub v2: Vertex,
    pub material_id: u32,
    pub node_index: usize,
}

impl Bounded for Triangle
{
    fn aabb(&self) -> AABB
    {
        let mut min = glm::vec3(std::f32::MAX, std::f32::MAX, std::f32::MAX);
        min = glm::min(min, self.v0.pos);
        min = glm::min(min, self.v1.pos);
        min = glm::min(min, self.v2.pos);

        let mut max = glm::vec3(-std::f32::MAX, -std::f32::MAX, -std::f32::MAX);
        max = glm::max(max, self.v0.pos);
        max = glm::max(max, self.v1.pos);
        max = glm::max(max, self.v2.pos);

        AABB::with_bounds(bvh::nalgebra::Point3::new(min.x, min.y, min.z), bvh::nalgebra::Point3::new(max.x, max.y, max.z))
    }
}

impl BHShape for Triangle
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

impl Shape for Triangle {
    fn intersect(&self, origin: glm::Vec3, direction: glm::Vec3, barry: &mut glm::Vec2) -> Option<f32>
    {
        let v0v1 = self.v1.pos - self.v0.pos;
        let v0v2 = self.v2.pos - self.v0.pos;
        let pvec = glm::cross(direction, v0v2);
        let det = glm::dot(v0v1, pvec);

        if det < 1e-8
        {
            return None;
        }
        /*if fabs(det) < 1e-8
        {
            return None;
        }*/

        let inv_det = 1f32 / det;

        let tvec = origin - self.v0.pos;
        let u = glm::dot(tvec, pvec) * inv_det;
        if u < 0f32 || u > 1f32
        {
            return None;
        }

        let qvec = glm::cross(tvec, v0v1);
        let v = glm::dot(direction, qvec) * inv_det;
        if v < 0f32 || u + v > 1f32
        {
            return None;
        }

        *barry = glm::vec2(u, v);

        return Some(glm::dot(v0v2, qvec) * inv_det);
    }

    fn get_normal(&self, _hit_pos: glm::Vec3, barry: glm::Vec2) -> glm::Vec3
    {
        return glm::normalize((self.v0.normal * (1f32 - barry.x - barry.y)) + (self.v1.normal * barry.x) + ( self.v2.normal * barry.y));
    }

    fn get_material_id(&self) -> u32
    {
        return self.material_id;
    }

    fn get_tangents(&self, _normal: glm::Vec3, tangent: &mut glm::Vec3, bitangent: &mut glm::Vec3, barry: glm::Vec2)
    {
        *tangent = glm::normalize((self.v0.tangent * (1f32 - barry.x - barry.y)) + (self.v1.tangent * barry.x) + ( self.v2.tangent * barry.y));
        *bitangent = glm::normalize((self.v0.bitangent * (1f32 - barry.x - barry.y)) + (self.v1.bitangent * barry.x) + ( self.v2.bitangent * barry.y));
    }
}