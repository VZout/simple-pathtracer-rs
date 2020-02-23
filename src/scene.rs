extern crate bvh;
extern crate glm;

use bvh::bvh::BVH;
use bvh::ray::Ray;
use crate::shape::*;
use crate::sphere::*;

pub struct Hit
{
    pub pos: glm::Vec3,
    pub normal: glm::Vec3,
    pub material_id: u32,
    pub time: f32,
}

pub struct SceneGraph
{
    pub objects: Vec<Sphere>,
    pub bvh: Option<BVH>,
}

impl SceneGraph
{
    pub fn add(&mut self, sphere: Sphere)
    {
        self.objects.push(sphere);
    }

    pub fn clear(&mut self)
    {
        self.objects.clear();
    }

    pub fn build(&mut self)
    {
        self.bvh = Some(BVH::build(&mut self.objects));
    }

    pub fn traverse(&self, origin: glm::Vec3, direction: glm::Vec3) -> Option<Hit>
    {
        let nalgebra_origin = bvh::nalgebra::Point3::new(origin.x, origin.y, origin.z);
        let nalgebra_direction = bvh::nalgebra::Vector3::new(direction.x, direction.y, direction.z);

        let ray = Ray::new(nalgebra_origin, nalgebra_direction);

        if let Some(bvh) = &self.bvh
        {
            let bb_hit_objects = bvh.traverse(&ray, &self.objects);

            if !bb_hit_objects.is_empty()
            {
                let mut closest_hit = None;
                let mut closest_t = std::f32::MAX;

                for obj in bb_hit_objects
                {
                    let t = obj.intersect(origin, direction);
                    if let Some(t) = t
                    {
                        if t <= closest_t
                        {
                            let hit_pos = origin + (direction * t);
                            let normal = obj.get_normal(hit_pos);
                            let material_id = obj.get_material_id();

                            closest_t = t;
                            closest_hit = Some(Hit
                            {
                                pos: hit_pos,
                                normal,
                                material_id,
                                time: t,
                            });
                        }
                    }
                }

                return closest_hit;
            }
        }

        return None;
    }
}