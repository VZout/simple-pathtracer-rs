extern crate sdl2;
extern crate rand;
extern crate crossbeam;
extern crate glm;

mod application;
mod scene;
mod sphere;
mod resource_manager;
mod shape;
mod material;

use crossbeam::thread;
use std::time::{Duration, Instant};
use application::*;
use scene::*;
use sphere::*;
use material::*;
use resource_manager::*;
use rand::Rng;
use std::f32::consts::PI;

struct Camera
{
    pos: glm::Vec3,
    aspect_ratio: f32,
    up: glm::Vec3,
    half_fov: f32,
    right: glm::Vec3,
    lens_radius: f32,
    forward: glm::Vec3,
    focal_dist: f32,
    viewport_width: u32,
    viewport_height: u32,
}

impl Copy for Camera {}
impl Clone for Camera {
    fn clone(&self) -> Self {
        *self
    }
}

struct MyApp
{
    start: Instant,
    fps_start: Instant,
    frame: u32,
    accumulation_idx: u32,
    camera: Camera,
    scene: SceneGraph,
    material_manager: MaterialManager,
}

fn camera_to_world(v: glm::Vec3, right: glm::Vec3, up: glm::Vec3, forward: glm::Vec3) -> glm::Vec3
{
    let v = glm::vec3(v.x, v.y * -1f32, v.z);
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
        let mut r = 0f32;
        let mut theta = 0f32;

        if up.x.abs() > up.y.abs()
        {
            r = up.x;
            theta = 0.25 * PI * (up.y / up.x);
        }
        else
        {
            r = up.y;
            theta = 0.5 * PI - 0.25 * PI * (up.x / up.y);
        }

        return glm::vec2(theta.cos(), theta.sin()) * r;
    }
}

fn generate_camera_ray(pixel_uv: glm::Vec2, lens_uv: glm::Vec2, p: &mut glm::Vec3, wo: &mut glm::Vec3, camera: &Camera)
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

#[allow(dead_code)]
fn next_rand() -> f32
{
    let mut rng = rand::thread_rng();
    return rng.gen_range(0f32, 1f32);
}

#[allow(dead_code)]
fn next_rand_v2() -> glm::Vec2
{
    let mut rng = rand::thread_rng();
    return glm::vec2(rng.gen_range(0f32, 1f32), rng.gen_range(0f32, 1f32));
}

fn parse_pixel(pixel: &mut Pixel, pos: glm::UVec2, camera: &Camera, scene: &SceneGraph, material_manager: &MaterialManager, accum_idx: u32)
{
    let pos_f = glm::vec2(pos.x as f32, pos.y as f32);
    let pixel_size = glm::vec2(1f32 / camera.viewport_width as f32, 1f32 / camera.viewport_height as f32);
    let pixel_pos = pos_f * pixel_size;

    let jitter = pixel_size * (next_rand_v2() - 0.5);
    let lens_uv  = next_rand_v2();
    let pixel_uv = pixel_pos + jitter;

    let mut origin = glm::vec3(0f32, 0f32, 0f32);
    let mut direction = glm::vec3(0f32, 0f32, 0f32);
    generate_camera_ray(pixel_uv, lens_uv, &mut origin, &mut direction, camera);

    let hit = scene.traverse(origin, direction);

    let mut color = glm::vec3(120f32 / 255f32, 190f32 / 255f32, 227f32 / 255f32);
    if let Some(hit) = &hit
    {
        let ambient = glm::vec3(0.1f32, 0.1f32, 0.1f32);

        let L = glm::normalize(glm::vec3(0f32, -1f32, -0.5f32));
        let diff = glm::max(glm::dot(hit.normal, L), 0.0);
        let diffuse = glm::vec3(1f32, 1f32, 1f32) * diff;

        let albedo = material_manager.get(&hit.material_id).unwrap().color;

        color = (ambient + diffuse) * albedo;
    }

    let prev = glm::vec3(pixel.r, pixel.g, pixel.b);
    let new = glm::vec3(color.x, color.y, color.z);
    let result = (prev * accum_idx as f32 + new) / (accum_idx as f32 + 1f32);

    pixel.r = result.x;
    pixel.g = result.y;
    pixel.b = result.z;
}

impl MyApp
{
    fn calc_fps(&mut self)
    {
        let duration = self.fps_start.elapsed();
        if duration >= Duration::new(1, 0)
        {
            println!("FPS: {:?}", self.frame);
            self.fps_start = Instant::now();
            self.frame = 0;
        }

        self.frame += 1;
        self.accumulation_idx += 1;
    }
}

impl Renderer for MyApp {
    fn init(&mut self, _app: &mut Application)
    {
        let _ = self.material_manager.place(&materials::RED);
        let _ = self.material_manager.place(&materials::GREEN);
        let _ = self.material_manager.place(&materials::BLUE);

        let mut rng = rand::thread_rng();
        let scene_width = 20f32;
        let scene_height = 20f32;

        for _ in 0..100
        {
            let material_id = rng.gen_range(0u32, 3u32);

            self.scene.add(Sphere
            {
                pos: glm::vec3(
                    rng.gen_range(-scene_width, scene_width),
                    rng.gen_range(-scene_height, scene_height),
                    rng.gen_range(20f32, 30f32)
                ),
                radius: 1f32,
                material_id,
                node_index: 0usize,
            });
        }

        self.scene.build();
    }

    fn render(&mut self, app: &mut Application)
    {
        //self.camera.pos.z = (self.start.elapsed().as_secs_f32()).sin() * 5f32 + 5f32;

        let num_pixels = app.back_buffer.width * app.back_buffer.height;
        let num_threads = 8;
        let thread_size = (num_pixels / num_threads) as usize;

        let bb_width = app.back_buffer.width.clone();
        let pixels = &mut app.back_buffer.pixels;
        let accum_idx = self.accumulation_idx;

        thread::scope(|s|
        {
            let pixels = &pixels; // shadowing the closures move...

            let mut handles = Vec::new();

            for thread_id in 0..num_threads
            {
                let start = thread_size * thread_id as usize;
                let camera = self.camera.clone();
                let scene = &self.scene;
                let material_maanger = &self.material_manager;

                let handle = s.spawn(move |_| unsafe
                {
                    for i in start..start + thread_size
                    {
                        let pixel_pos = glm::uvec2(
                            i as u32 % bb_width,
                            (i as f32 / bb_width as f32).floor() as u32
                        );

                        // To avoid the borrow checker I use pointers here so I can access 1 array from multiple threads...
                        let pixel = (pixels.as_ptr() as *mut Pixel).offset(i as isize);
                        parse_pixel(&mut *pixel, pixel_pos, &camera, &scene, &material_maanger, accum_idx);
                    }
                });

                handles.push(handle)
            }

            for handle in handles
            {
                handle.join().unwrap();
            }
        }).unwrap();

       /*
       let mut pixel_pos = glm::uvec2( 0u32, 0u32 );
        for pixel in pixels
        {
            parse_pixel(pixel, pixel_pos, &self.camera, &self.scene, &self.material_manager, self.accumulation_idx);

            pixel_pos.x += 1;
            if pixel_pos.x >= app.back_buffer.width
            {
                pixel_pos.x = 0;
                pixel_pos.y += 1;
            }
        }
        */

        self.calc_fps();
    }
}

fn main()
{
    let back_buffer_width = 600;
    let back_buffer_height = 600;

    let fov = 80f32;
    let aspect_ratio = back_buffer_width as f32 / back_buffer_height as f32;
    let lens_dim = 2f32;
    let focal_dist = 20f32;

    let camera = Camera
    {
        pos: glm::vec3(0f32, 0f32, 0f32),
        aspect_ratio,
        up: glm::vec3(0f32, 1f32, 0f32),
        half_fov: (0.5f32 * fov.to_radians()).tan(),
        right: glm::vec3(1f32, 0f32, 0f32),
        lens_radius: 0.5f32 * lens_dim,
        forward: glm::vec3(0f32, 0f32, 1f32),
        focal_dist,
        viewport_width: back_buffer_width,
        viewport_height: back_buffer_height,
    };

    let scene = SceneGraph
    {
        objects: Vec::new(),
        bvh: None,
    };

    let now = Instant::now();

    let mut app = MyApp{ start: now, fps_start: now, frame: 0, accumulation_idx: 0, camera, scene, material_manager: MaterialManager::new() };

    AppBuilder::new("My Raytracer", back_buffer_width, back_buffer_height)
        .start(&mut app);
}
