use std::f32::consts::{FRAC_PI_4, FRAC_PI_2};

pub struct Camera
{
    pub pos: glm::Vec3,
    pub aspect_ratio: f32,
    pub up: glm::Vec3,
    pub half_fov: f32,
    pub right: glm::Vec3,
    pub lens_radius: f32,
    pub forward: glm::Vec3,
    pub focal_dist: f32,
    pub viewport_width: u32,
    pub viewport_height: u32,
}

impl Copy for Camera {}
impl Clone for Camera {
    fn clone(&self) -> Self {
        *self
    }
}

fn camera_to_world(v: glm::Vec3, right: glm::Vec3, up: glm::Vec3, forward: glm::Vec3) -> glm::Vec3
{
    let v = glm::vec3(v.x, v.y, v.z);
    return right * v.x + up * v.y + forward * v.z;
}

fn sample_disk_concentric(u: glm::Vec2) -> glm::Vec2
{
    let up = (u * 2f32) - glm::vec2(1f32, 1f32);
    if up == glm::vec2(0f32, 0f32)
    {
        return glm::vec2(0f32, 0f32);
    }
    else
    {
        //if up.x.abs() > up.y.abs()
        if (up.x * up.x) > (up.y * up.y) // slightly faster than absolute values
        {
            let r = up.x;
            let theta = FRAC_PI_4 * (up.y / up.x);

            return glm::vec2(theta.cos(), theta.sin()) * r;
        }
        else
        {
            let r = up.y;
            let theta = FRAC_PI_2 - FRAC_PI_4 * (up.x / up.y);

            return glm::vec2(theta.cos(), theta.sin()) * r;
        }
    }
}

pub fn generate_camera_ray(pixel_uv: glm::Vec2, lens_uv: glm::Vec2, p: &mut glm::Vec3, wo: &mut glm::Vec3, camera: &Camera)
{
    let up = camera.up;
    let right = camera.right;
    let forward = camera.forward;

    let tx = camera.half_fov * camera.aspect_ratio * (2f32 * pixel_uv.x - 1f32);
    let ty = camera.half_fov * (2f32 * pixel_uv.y - 1f32);

    let mut p_camera  = glm::vec3(0f32, 0f32, 0f32);
    let mut wo_camera = glm::normalize(glm::vec3(tx, ty, 1.0));

    // Depth of Field
    if camera.lens_radius > 0.0
    {
        let t_focus = camera.focal_dist / wo_camera.z;
        let p_focus = wo_camera * t_focus;
        let temp = sample_disk_concentric(lens_uv) * camera.lens_radius;
        p_camera.x = temp.x;
        p_camera.y = temp.y;
        wo_camera = glm::normalize(p_focus - p_camera);
    }

    *p  = camera_to_world(p_camera, right, up, forward) + camera.pos;
    *wo = camera_to_world(wo_camera, right, up, forward);
}