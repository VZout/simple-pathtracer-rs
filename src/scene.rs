extern crate bvh;
extern crate glm;

use bvh::bvh::BVH;
use bvh::ray::Ray;
use crate::shape::*;
use crate::triangle::*;
use crate::model::Vertex;

pub struct Hit
{
    pub pos: glm::Vec3,
    pub normal: glm::Vec3,
    pub tangent: glm::Vec3,
    pub bitangent: glm::Vec3,
    pub uv: glm::Vec2,
    pub material_id: u32,
    pub time: f32,
}

pub struct SceneGraph
{
    pub triangles: Vec<Triangle>,
    pub bvh: Option<BVH>,
}

impl SceneGraph
{
    pub fn add_tri(&mut self, v0: Vertex, v1: Vertex, v2: Vertex, material_id: u32)
    {
        self.triangles.push(Triangle {
            v0,
            v1,
            v2,
            material_id,
            node_index: 0usize,
        });
    }

    #[allow(dead_code)]
    pub fn clear(&mut self)
    {
        self.triangles.clear();
    }

    pub fn build(&mut self)
    {
        let bvh = BVH::build(&mut self.triangles);
        self.bvh = Some(bvh);
    }

    pub fn traverse(&self, origin: glm::Vec3, direction: glm::Vec3) -> Option<Hit>
    {

        let nalgebra_origin = bvh::nalgebra::Point3::new(origin.x, origin.y, origin.z);
        let nalgebra_direction = bvh::nalgebra::Vector3::new(direction.x, direction.y, direction.z);
        let ray = Ray::new(nalgebra_origin, nalgebra_direction);

        if let Some(bvh) = &self.bvh
        {
            let bb_hit_objects = bvh.traverse(&ray, &self.triangles);

            if !bb_hit_objects.is_empty()
            {
                let mut closest_obj = None;
                let mut closest_t = std::f32::MAX;
                let mut closest_barry = glm::vec2(0f32, 0f32);

                for obj in &bb_hit_objects
                {
                    let mut barry = glm::vec2(0f32, 0f32);
                    let t = obj.intersect(origin, direction, &mut barry);
                    if let Some(t) = t
                    {
                        if t <= closest_t && t > 0f32
                        {
                            closest_t = t;
                            closest_obj = Some(obj);
                            closest_barry = barry;
                        }
                    }
                }

                if let Some(closest_obj) = closest_obj
                {
                    let mut tangent = glm::vec3(0f32, 0f32, 0f32);
                    let mut bitangent = glm::vec3(0f32, 0f32, 0f32);

                    let hit_pos = origin + (direction * closest_t);
                    let normal = closest_obj.get_normal(hit_pos, closest_barry);
                    let material_id = closest_obj.get_material_id();
                    let uv = closest_obj.get_uv(closest_barry);
                    closest_obj.get_tangents(normal, &mut tangent, &mut bitangent, closest_barry);

                    return Some(Hit
                    {
                        pos: hit_pos,
                        normal,
                        tangent,
                        bitangent,
                        uv,
                        material_id,
                        time: closest_t,
                    });
                }
            }
        }

        return None;
    }

    pub fn tri_count(&self) -> usize
    {
        return self.triangles.len();
    }

    pub fn traverse_shadow(&self, origin: glm::Vec3, direction: glm::Vec3, max_distance: f32) -> bool
    {
        let nalgebra_origin = bvh::nalgebra::Point3::new(origin.x, origin.y, origin.z);
        let nalgebra_direction = bvh::nalgebra::Vector3::new(direction.x, direction.y, direction.z);
        let ray = Ray::new(nalgebra_origin, nalgebra_direction);

        if let Some(bvh) = &self.bvh
        {
            let bb_hit_objects = bvh.traverse(&ray, &self.triangles);

            if !bb_hit_objects.is_empty()
            {
                for obj in &bb_hit_objects
                {
                    let mut barry = glm::vec2(0f32, 0f32);
                    let t = obj.intersect(origin, direction, &mut barry);

                    if let Some(t) = t
                    {
                        if t <= max_distance
                        {
                            return true;
                        }
                    }
                }
            }
        }

        return false;
    }
}