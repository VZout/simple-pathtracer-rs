extern crate rand;

use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng as RandGenerator;

#[allow(dead_code)]
pub fn create_rand_generator() -> RandGenerator
{
    return rand::rngs::SmallRng::from_entropy()
}

#[allow(dead_code)]
pub fn rand_f32_r(min: f32, max: f32, rng: &mut RandGenerator) -> f32
{
    return rng.gen_range(min, max);
}

#[allow(dead_code)]
pub fn rand_u32_r(min: u32, max: u32, rng: &mut RandGenerator) -> u32
{
    return rng.gen_range(min, max);
}

#[allow(dead_code)]
pub fn next_rand(rng: &mut RandGenerator) -> f32
{
    return rng.gen_range(0f32, 1f32);
}

#[allow(dead_code)]
pub fn next_rand_v2(rng: &mut RandGenerator) -> glm::Vec2
{
    return glm::vec2(rng.gen_range(0f32, 1f32), rng.gen_range(0f32, 1f32));
}