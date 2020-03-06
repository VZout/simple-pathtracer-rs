extern crate sdl2;
extern crate glm;
extern crate oidn;
extern crate image;

mod application;
mod scene;
mod resource_manager;
mod shape;
mod material;
mod model;
mod triangle;
mod random;
mod disney;
mod texture;
mod camera;

use rand::rngs::SmallRng as RandGenerator;
use std::time::{Duration, Instant};
use application::*;
use model::*;
use random::*;
use scene::*;
use texture::*;
use material::*;
use camera::*;
use rayon::prelude::*;
use disney::*;
use image::GenericImageView;

static EPSILON: f32 = 0.0001f32;
static ACCUMULATE: bool = true;
static JITTER_AA:bool = true;
static SHOW_FPS: bool = false;
static NUM_THREADS: u32 = 8;
static RAY_DEPTH: u32 = 3;
static DENOISE: bool = true;
static GBUFFER_SAMPLES: u32 = 300;
static SAMPLES_BEFORE_DENOISE: u32 = 100;
static USE_EXTENDED_DENOISING: bool = true;

struct MyApp
{
    fps_start: Instant,
    app_start: Instant,
    frame: u32,
    accumulation_idx: u32,
    camera: Camera,
    scene: SceneGraph,
    material_manager: MaterialManager,
    texture_manager: TextureManager,
    model_manager: ModelManager,
}

fn calculate_gbuffers(albedo: &mut Pixel, normal: &mut Pixel, pos: glm::Vec2, camera: &Camera, scene: &SceneGraph, material_manager: &MaterialManager, texture_manager: &TextureManager, accum_idx: u32, rng: &mut RandGenerator)
{
    let pixel_size = glm::vec2(1f32 / camera.viewport_width as f32, 1f32 / camera.viewport_height as f32);
    let pixel_pos = pos * pixel_size;

    let jitter = if JITTER_AA { pixel_size * (next_rand_v2(rng) - 0.5)} else { glm::vec2(0f32, 0f32) };
    let lens_uv  = next_rand_v2(rng);
    let pixel_uv = pixel_pos + jitter;

    let mut origin = glm::vec3(0f32, 0f32, 0f32);
    let mut direction = glm::vec3(0f32, 0f32, 0f32);
    generate_camera_ray(pixel_uv, lens_uv, &mut origin, &mut direction, camera);

    let mut ray_albedo = glm::vec3(0f32, 0f32, 0f32);
    let mut ray_normal = glm::vec3(0f32, 0f32, 0f32);

    let hit = scene.traverse(origin, direction);

    if let Some(hit) = &hit
    {
        let mut material = *material_manager.get(&hit.material_id).unwrap().clone();

        ray_albedo = material.color;
        ray_normal = hit.normal;
    }
    else
    {
        ray_albedo = glm::vec3(0.7f32, 0.7f32, 0.7f32);
    }

    let prev_albedo = pixel_to_vec3(albedo);
    let albedo_result = (prev_albedo * accum_idx as f32 + ray_albedo) / (accum_idx as f32 + 1f32);
    *albedo = vec3_to_pixel(&albedo_result);

    let prev_normal = pixel_to_vec3(normal);
    let normal_result = (prev_normal * accum_idx as f32 + ray_normal) / (accum_idx as f32 + 1f32);
    *normal = vec3_to_pixel(&normal_result);
}

fn parse_pixel(pixel: &mut Pixel, pos: glm::Vec2, camera: &Camera, scene: &SceneGraph, material_manager: &MaterialManager, texture_manager: &TextureManager, accum_idx: u32, rng: &mut RandGenerator)
{
    let pixel_size = glm::vec2(1f32 / camera.viewport_width as f32, 1f32 / camera.viewport_height as f32);
    let pixel_pos = pos * pixel_size;

    let jitter = if JITTER_AA { pixel_size * (next_rand_v2(rng) - 0.5)} else { glm::vec2(0f32, 0f32) };
    let lens_uv  = next_rand_v2(rng);
    let pixel_uv = pixel_pos + jitter;

    let mut origin = glm::vec3(0f32, 0f32, 0f32);
    let mut direction = glm::vec3(0f32, 0f32, 0f32);
    generate_camera_ray(pixel_uv, lens_uv, &mut origin, &mut direction, camera);

    let mut ray_color = glm::vec3(0f32, 0f32, 0f32);
    let mut throughput = glm::vec3(1f32, 1f32, 1f32);

    'recursive_trace: for _ in 0..RAY_DEPTH
    {
        let hit = scene.traverse(origin, direction);
        let v = -direction;

        if let Some(hit) = &hit
        {
            let material = get_surface_material(&hit, &material_manager, &texture_manager);

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
                break 'recursive_trace;
            }

            ray_color = glm::clamp(color, glm::vec3(0f32, 0f32, 0f32), glm::vec3(2f32, 2f32, 2f32));
            origin = hit.pos + (bsdf_dir * EPSILON);
            direction = bsdf_dir;
        }
        else
        {
            ray_color = glm::clamp(throughput * glm::vec3(0.7f32, 0.7f32, 0.7f32), glm::vec3(0f32, 0f32, 0f32), glm::vec3(2f32, 2f32, 2f32));
            break 'recursive_trace;
        }
    }

    let prev = pixel_to_vec3(pixel);
    let result = (prev * accum_idx as f32 + ray_color) / (accum_idx as f32 + 1f32);
    *pixel = vec3_to_pixel(&result);
}

impl MyApp
{
    fn calc_fps(&mut self)
    {
        if SHOW_FPS
        {
            let duration = self.fps_start.elapsed();
            if duration >= Duration::new(1, 0)
            {
                println!("FPS: {:?}", self.frame);
                self.fps_start = Instant::now();
                self.frame = 0;
            }

            self.frame += 1;
        }

        if ACCUMULATE
        {
            self.accumulation_idx += 1;
        }
    }
}

impl Renderer for MyApp
{
    fn init(&mut self, app: &mut Application)
    {
        rayon::ThreadPoolBuilder::new().num_threads(NUM_THREADS as usize).build_global().unwrap();

        let mat_0 = self.material_manager.load(&materials::GLOSSY_WHITE);
        let _ = self.material_manager.load(&materials::GREEN);
        let _ = self.material_manager.load(&materials::GLOSSY_ORANGE);
        let _ = self.material_manager.load(&materials::BLUE);

        load_textures_of_material(&mat_0, &mut self.material_manager, &mut self.texture_manager);

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

        println!("#################################");
        println!("Thread Count: {}", NUM_THREADS);
        println!("Rays per frame: {}", app.back_buffer.width * app.back_buffer.height * RAY_DEPTH);
        println!("Bounces: {}", RAY_DEPTH - 1);
        println!("Triangles: {}", self.scene.tri_count());
        println!("#################################");

        self.scene.build();
    }

    fn render(&mut self, app: &mut Application)
    {
        let num_pixels = app.back_buffer.width * app.back_buffer.height;

        let bb_width = app.back_buffer.width.clone();
        let pixels = &mut app.back_buffer.pixels;

        (0..num_pixels).into_par_iter().for_each(|i| unsafe
        {
            let mut rng = create_rand_generator();

            let pixel_pos = glm::vec2(
                i as f32 % bb_width as f32,
                (i as f32 / bb_width as f32).floor()
            );

            let pixel = (pixels.as_ptr() as *mut Pixel).offset(i as isize);
            parse_pixel(&mut *pixel, pixel_pos, &self.camera, &self.scene, &self.material_manager, &self.texture_manager, self.accumulation_idx, &mut rng);
        });

        self.calc_fps();

        // image denoising
        if DENOISE
        {
            if (self.accumulation_idx % 25) == 0 && self.accumulation_idx <= SAMPLES_BEFORE_DENOISE
            {
                let elapsed = self.fps_start.elapsed().as_secs_f32();
                let percent = self.accumulation_idx as f32 / SAMPLES_BEFORE_DENOISE as f32;
                let remaining = elapsed * (1f32 / percent - 1f32);
                println!("({:.2}%) \t Elapsed: {:.2} min \t ETA: {:.2} min", percent * 100f32, elapsed / 60f32, remaining / 60f32);
            }

            if self.accumulation_idx == SAMPLES_BEFORE_DENOISE
            {
                println!("Accumulation Finished");

                // Get GBUFFERS
                let mut albedo_pixels = vec![Pixel { r: 0f32, g: 0f32, b: 0f32, a: 0f32 }; (app.back_buffer.width * app.back_buffer.height) as usize];
                let mut normal_pixels = vec![Pixel { r: 0f32, g: 0f32, b: 0f32, a: 0f32 }; (app.back_buffer.width * app.back_buffer.height) as usize];

                if USE_EXTENDED_DENOISING
                {
                    println!("Generating GBuffers....");

                    for i in 0..GBUFFER_SAMPLES
                    {
                        (0..num_pixels).into_par_iter().for_each(|i| unsafe
                        {
                            let mut rng = create_rand_generator();

                            let pixel_pos = glm::vec2(
                                i as f32 % bb_width as f32,
                                (i as f32 / bb_width as f32).floor()
                            );

                            let albedo = (albedo_pixels.as_ptr() as *mut Pixel).offset(i as isize);
                            let normal = (normal_pixels.as_ptr() as *mut Pixel).offset(i as isize);
                            calculate_gbuffers(&mut *albedo, &mut *normal, pixel_pos, &self.camera, &self.scene, &self.material_manager, &self.texture_manager, self.accumulation_idx, &mut rng);
                        });
                    }

                    println!("Finished Generating GBuffers");
                }

                let input_img = f32vec_from_pixels(&app.back_buffer.pixels);
                let input_albedo = f32vec_from_pixels(&albedo_pixels);
                let input_normal = f32vec_from_pixels(&normal_pixels);

                let mut filter_output = vec![0.0f32; input_img.len()];

                let device = oidn::Device::new();
                let mut filter = oidn::RayTracing::new(&device);

                filter.set_srgb(true);
                filter.set_img_dims(app.back_buffer.width as usize, app.back_buffer.height as usize);
                if USE_EXTENDED_DENOISING
                {
                    filter.set_albedo(&input_albedo[..]);
                    filter.set_normal(&input_normal[..]);
                }
                filter.execute(&input_img[..], &mut filter_output[..]).expect("Filter config error!");

                if let Err(e) = device.get_error() {
                    println!("Error denosing image: {}", e.1);
                } else {
                    // Save denoised image
                    let mut out_denoised = Vec::new();
                    for data in filter_output
                    {
                        out_denoised.push((data.powf(1f32 / 2.2) * 65535f32) as u16);
                    }
                    let denoised_u8 = unsafe { std::slice::from_raw_parts(out_denoised.as_ptr() as *mut u8, out_denoised.len() * 2) };
                    image::save_buffer(&std::path::Path::new("image_denoised.png"), &denoised_u8[..], app.back_buffer.width, app.back_buffer.height, image::ColorType::Rgb16);

                    let dir = std::env::current_dir().unwrap().to_str().unwrap().replace("\\", "/");
                    println!("Saved Denoised File: file:///{}/{}", dir, "image_denoised.png");

                    save_pixels("image.png", &app.back_buffer.pixels, app.back_buffer.width, app.back_buffer.height);
                    //save_pixels("g_albedo.png", &albedo_pixels, app.back_buffer.width, app.back_buffer.height);
                    //save_pixels("g_normal.png", &normal_pixels, app.back_buffer.width, app.back_buffer.height);

                    let mut img_denoised = image::open("image_denoised.png").unwrap();
                    img_denoised = img_denoised.flipv();
                    img_denoised.save("image_denoised.png");
                }
            }
        }
    }
}

fn save_pixels(path: &str, pixels: &Vec<Pixel>, width: u32, height: u32)
{
    // Save non-denoised image
    let mut vec_u16 = Vec::new();
    for pixel in pixels
    {
        vec_u16.push((pixel.r.powf(1f32 / 2.2) * 65535f32).min(65535f32) as u16);
        vec_u16.push((pixel.g.powf(1f32 / 2.2) * 65535f32).min(65535f32) as u16);
        vec_u16.push((pixel.b.powf(1f32 / 2.2) * 65535f32).min(65535f32) as u16);
    }
    let data_u8 = unsafe { std::slice::from_raw_parts(vec_u16.as_ptr() as *mut u8, vec_u16.len() * 2) };

    image::save_buffer(&std::path::Path::new(path), &data_u8[..], width, height, image::ColorType::Rgb16).unwrap();

    let mut img_original = image::open(path).unwrap();
    img_original = img_original.flipv();
    img_original.save(path);

    let dir = std::env::current_dir().unwrap().to_str().unwrap().replace("\\", "/");
    println!("Saved File: file:///{}/{}", dir, path);
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
        right: glm::vec3(-1f32, 0f32, 0f32),
        lens_radius: 0.5f32 * lens_dim,
        forward: glm::vec3(0f32, 0f32, 1f32),
        focal_dist,
        viewport_width: back_buffer_width,
        viewport_height: back_buffer_height,
    };

    let scene = SceneGraph
    {
        triangles: Vec::new(),
        bvh: None,
    };

    let now = Instant::now();

    let mut app = MyApp{
        fps_start: now,
        app_start: now,
        frame: 0,
        accumulation_idx: 0,
        camera,
        scene,
        material_manager: MaterialManager::new(MaterialLoader{}),
        texture_manager: TextureManager::new(TextureLoader{}),
        model_manager: ModelManager::new(ModelLoader{})
    };

    AppBuilder::new("My Raytracer", back_buffer_width, back_buffer_height)
        .start(&mut app);
}
