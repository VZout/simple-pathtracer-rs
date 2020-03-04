extern crate sdl2;
extern crate crossbeam;
extern crate glm;

mod application;
mod scene;
mod sphere;
mod resource_manager;
mod shape;
mod material;
mod model;
mod triangle;
mod random;
mod disney;
mod camera;

use rand::rngs::SmallRng as RandGenerator;
use crossbeam::thread;
use std::time::{Duration, Instant};
use application::*;
use model::*;
use random::*;
use scene::*;
use sphere::*;
use material::*;
use camera::*;
use rayon::prelude::*;
use crate::disney::*;

static NUM_THREADS: u32 = 8;

struct MyApp
{
    fps_start: Instant,
    frame: u32,
    accumulation_idx: u32,
    camera: Camera,
    scene: SceneGraph,
    material_manager: MaterialManager,
    model_manager: ModelManager,
}

fn parse_pixel(pixel: &mut Pixel, pos: glm::Vec2, camera: &Camera, scene: &SceneGraph, material_manager: &MaterialManager, accum_idx: u32, rng: &mut RandGenerator)
{
    let pixel_size = glm::vec2(1f32 / camera.viewport_width as f32, 1f32 / camera.viewport_height as f32);
    let pixel_pos = pos * pixel_size;

    let jitter = pixel_size * (next_rand_v2(rng) - 0.5);
    let lens_uv  = next_rand_v2(rng);
    let pixel_uv = pixel_pos + jitter;

    let mut origin = glm::vec3(0f32, 0f32, 0f32);
    let mut direction = glm::vec3(0f32, 0f32, 0f32);
    generate_camera_ray(pixel_uv, lens_uv, &mut origin, &mut direction, camera);

    let max_depth = 3;
    let mut ray_color = glm::vec3(0f32, 0f32, 0f32);
    let mut throughput = glm::vec3(1f32, 1f32, 1f32);

    for _ in 0..max_depth
    {
        let hit = scene.traverse(origin, direction);
        let v = -direction;

        if let Some(hit) = &hit
        {
            let material = material_manager.get(&hit.material_id).unwrap();

            //let color = disney::direct_lighting(&scene, &hit, &material, v) * throughput;
            let color = glm::vec3(0f32, 0f32, 0f32);

            let bsdf_dir = disney::sample(&hit, &material, v, rng);

            let l = bsdf_dir;
            let h = glm::normalize(v + l);

            let n_dot_v = glm::dot(hit.normal, v).abs().max(MIN_N_DOT_V);
            let n_dot_l = glm::dot(hit.normal, l).abs();
            let n_dot_h = glm::dot(hit.normal, h).abs();
            let l_dot_h = glm::dot(l, h).abs();

            let pdf = disney::pdf(&hit, &material, v, l);
            if pdf > 0f32
            {
                let occlusion = (!(n_dot_l <= 0f32 || n_dot_v <= 0f32)) as i32 as f32;
                throughput = throughput * ((disney::evaluate(&material, n_dot_l, n_dot_v, n_dot_h, l_dot_h) * occlusion) / pdf);
            }
            else
            {
                break;
            }

            ray_color = color;
            origin = hit.pos;
            direction = bsdf_dir;
        }
        else
        {
            ray_color = throughput * glm::vec3(0.7f32, 0.7f32, 0.7f32);
            break;
        }
    }

    let prev = glm::vec3(pixel.r, pixel.g, pixel.b);
    let new = glm::vec3(ray_color.x, ray_color.y, ray_color.z);
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

impl Renderer for MyApp
{
    fn init(&mut self, _app: &mut Application)
    {
        rayon::ThreadPoolBuilder::new().num_threads(NUM_THREADS as usize).build_global().unwrap();

        let _ = self.material_manager.place(materials::GLOSSY_WHITE);
        let _ = self.material_manager.place(materials::GREEN);
        let _ = self.material_manager.place(materials::GLOSSY_ORANGE);
        let _ = self.material_manager.place(materials::BLUE);

        let mut rng = create_rand_generator();
        let scene_width = 20f32;
        let scene_height = 20f32;

        for _ in 0..100
        {
            let material_id = rand_u32_r(0u32, 3u32, &mut rng);

            self.scene.add_sphere(Sphere
            {
                pos: glm::vec3(
                    rand_f32_r(-scene_width, scene_width, &mut rng),
                    rand_f32_r(-scene_height, scene_height, &mut rng),
                    rand_f32_r(20f32, 30f32, &mut rng)
                ),
                radius: 1f32,
                material_id,
                node_index: 0usize,
            });
        }

        let model_handle = self.model_manager.load("test.fbx");
        let model = self.model_manager.get(&model_handle);

        let model_matrix = glm::mat3(
            1f32, 0f32, 0f32,
            0f32, 1f32, 0f32,
            0f32, 0f32, 1f32,
        );

        let mut material_id = 0u32;
        for mesh in &model.unwrap().meshes
        {
            for i in (0..mesh.indices.len()).step_by(3)
            {
                let mut v0 = mesh.vertices[mesh.indices[i + 0] as usize];
                let mut v1 = mesh.vertices[mesh.indices[i + 1] as usize];
                let mut v2 = mesh.vertices[mesh.indices[i + 2] as usize];

                v0.pos = model_matrix * v0.pos;
                v1.pos = model_matrix * v1.pos;
                v2.pos = model_matrix * v2.pos;

                self.scene.add_tri(v0, v1, v2, material_id);
            }

            material_id += 1;
        }

        self.scene.build();
    }

    fn render(&mut self, app: &mut Application)
    {
        let num_pixels = app.back_buffer.width * app.back_buffer.height;
        let thread_size = (num_pixels / NUM_THREADS) as usize;

        let bb_width = app.back_buffer.width.clone();
        let pixels = &mut app.back_buffer.pixels;
        let accum_idx = self.accumulation_idx;

        /*
        thread::scope(|s|
        {
            let pixels = &pixels; // shadowing the closures move...

            let mut handles = Vec::new();

            for thread_id in 0..num_threads
            {
                let start = thread_size * thread_id as usize;
                let camera = self.camera.clone();
                let scene = &self.scene;
                let material_manger = &self.material_manager;

                let handle = s.spawn(move |_| unsafe
                {
                    let mut rng = create_rand_generator();

                    for i in start..start + thread_size
                    {
                        let pixel_pos = glm::vec2(
                            i as f32 % bb_width as f32,
                            (i as f32 / bb_width as f32).floor()
                        );

                        // To avoid the borrow checker I use pointers here so I can access 1 array from multiple threads...
                        let pixel = (pixels.as_ptr() as *mut Pixel).offset(i as isize);
                        parse_pixel(&mut *pixel, pixel_pos, &camera, &scene, &material_manger, accum_idx, &mut rng);
                    }
                });

                handles.push(handle)
            }

            for handle in handles
            {
                handle.join().unwrap();
            }
        }).unwrap();
        */

        ///* Rayon
        (0..num_pixels).into_par_iter().for_each(|i| unsafe
        {
            let mut rng = create_rand_generator();

            let pixel_pos = glm::vec2(
                i as f32 % bb_width as f32,
                (i as f32 / bb_width as f32).floor()
            );

            let pixel = (pixels.as_ptr() as *mut Pixel).offset(i as isize);
            parse_pixel(&mut *pixel, pixel_pos, &self.camera, &self.scene, &self.material_manager, self.accumulation_idx, &mut rng);
        });

        self.calc_fps();
    }
}

// glam -> reciprocal???
fn main()
{
    let back_buffer_width = 600;
    let back_buffer_height = 600;

    let fov = 45f32;
    let aspect_ratio = back_buffer_width as f32 / back_buffer_height as f32;
    let lens_dim = 0f32;
    let focal_dist = 10f32;

    let camera = Camera
    {
        pos: glm::vec3(0f32, 0f32, -10f32),
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
        spheres: Vec::new(),
        triangles: Vec::new(),
        bvh: None,
    };

    let now = Instant::now();

    let mut app = MyApp{
        fps_start: now,
        frame: 0,
        accumulation_idx: 0,
        camera,
        scene,
        material_manager: MaterialManager::new(MaterialLoader{}),
        model_manager: ModelManager::new(ModelLoader{})
    };

    AppBuilder::new("My Raytracer", back_buffer_width, back_buffer_height)
        .start(&mut app);
}
