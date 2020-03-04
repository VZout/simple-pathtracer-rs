use crate::scene::*;
use rand::rngs::SmallRng as RandGenerator;
use crate::material::*;
use std::f32::consts::{ FRAC_PI_2, PI};
use crate::random::{next_rand, next_rand_v2};

static TWO_PI: f32 = 6.283185307179586476925286766559;

pub static MIN_N_DOT_V: f32 = 1e-4;

pub fn reflect(i: glm::Vec3, n: glm::Vec3) -> glm::Vec3
{
    return i - n * 2f32 * glm::dot(i, n);
}

#[allow(dead_code)]
fn shadow_terminator_term_chiang2019(hit: &Hit, l: glm::Vec3) -> f32
{
    let geometric_n = hit.normal;

    let n_dot_l = glm::dot(hit.normal, l).max(0f32);
    let g_n_dot_l = glm::dot(geometric_n, l).max(0f32);
    let g_n_dot_n = glm::dot(geometric_n, hit.normal).max(0f32);

    if n_dot_l.abs() < 0f32 || g_n_dot_l.abs() < 0f32 || g_n_dot_n.abs() < 0f32
    {
        return 0f32;
    }
    else
    {
        let g = g_n_dot_l / (n_dot_l * g_n_dot_n);
        if g <= 1f32
        {
            let smooth_term = -(g * g * g) + (g * g) + g; // smoothTerm is G' in the math
            return smooth_term;
        }
    }
    return 1f32;
}

fn cos_hemisphere_sample(u: glm::Vec2) -> glm::Vec3
{
    let r = u.x.sqrt();
    let phi = FRAC_PI_2 * u.y;
    let mut dir = glm::vec3(
        r * phi.cos(),
        r * phi.sin(),
        0f32,
    );
    dir.z = (1.0 - dir.x * dir.x - dir.y * dir.y).max(0.0001f32).sqrt();
    return dir;
}

fn spherical_direction(sin_theta: f32, cos_theta: f32, sin_phi: f32, cos_phi: f32) -> glm::Vec3
{
    return glm::vec3(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta);
}

fn lambert() -> f32
{
    return 1f32 / PI;
}

// Aproximate Luminance
fn luminance(rgb: glm::Vec3) -> f32
{
    return glm::dot(rgb, glm::vec3(0.2126f32, 0.7152f32, 0.0722f32));
}

fn diffuse(material: &Material) -> glm::Vec3
{
    return material.color * lambert();
}

fn gtr2(n_dot_h: f32, a: f32) -> f32
{
    let a2 = a * a;
    let t = 1.0 + (a2 - 1.0) * n_dot_h * n_dot_h;
    return a2 / (PI * t * t);
}

fn schlick_fresnel_reflectance(u: f32) -> f32
{
    let m = glm::clamp(1f32 - u, 0f32, 1f32);
    let m2 = m * m;
    return m2 * m2 * m;
}

fn g_schlick_smith_ggx(n_dot_l: f32, n_dot_v: f32, roughness: f32) -> f32
{
    // Heitz 2014, "Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs"
    let a2 = roughness * roughness;
    // TODO: lambdaV can be pre-computed for all the lights, it should be moved out of this function
    let lambda_v = n_dot_l * ((n_dot_v - a2 * n_dot_v) * n_dot_v + a2).sqrt();
    let lambda_l = n_dot_v * ((n_dot_l - a2 * n_dot_l) * n_dot_l + a2).sqrt();
    let v = 0.5 / (lambda_v + lambda_l);
    // a2=0 => v = 1 / 4*NoL*NoV   => min=1/4, max=+inf
    // a2=1 => v = 1 / 2*(NoL+NoV) => min=1/4, max=+inf
    // clamp to the maximum value representable in mediump
    return v;
}

// Geometric Shadowing function optimized (Perf > Quality)
#[allow(dead_code)]
fn g_schlick_smith_ggx_fast(n_dot_l: f32, n_dot_v: f32, roughness: f32) -> f32
{
    let v = 0.5 / glm::mix(2.0 * n_dot_l * n_dot_v, n_dot_l + n_dot_v, roughness);
    return v;
}

// Microfacet Isotropic
fn specular_isotropic(material: &Material, n_dot_l: f32, n_dot_v: f32, n_dot_h: f32, l_dot_h: f32) -> glm::Vec3
{
    let specular_tint = glm::vec3(1f32, 1f32, 1f32);

    let cdlum = luminance(material.color);
    let ctint = if cdlum > 0f32
    {
        material.color / cdlum
    }
    else
    {
        glm::vec3(1f32, 1f32, 1f32)
    };
    let cspec0 = glm::mix(glm::mix(glm::vec3(1f32, 1f32, 1f32), ctint, specular_tint) * material.specular * 0.08f32, material.color, glm::vec3(material.metallic, material.metallic, material.metallic)); // TODO: This is f0?
    let a = (material.roughness * material.roughness).max(0.001f32); // Make a function for this.

    let fh = schlick_fresnel_reflectance(l_dot_h);
    let d = gtr2(n_dot_h, a);
    let f = glm::mix(cspec0, glm::vec3(1f32, 1f32, 1f32), glm::vec3(fh, fh, fh));
    let g = g_schlick_smith_ggx(n_dot_l, n_dot_v, a);

    return f * g * d;
}

fn same_hemisphere(hit: &Hit, v: glm::Vec3, l: glm::Vec3) -> bool
{
    return glm::dot(v, hit.normal) * glm::dot(l, hit.normal) > 0.0; // TODO: Duplicate dot products
}

// Lambartian Reflection
fn pdf_diffuse(hit: &Hit, l: glm::Vec3) -> f32
{
    return glm::dot(hit.normal, l).abs() / PI;
}

fn sample_diffuse(hit: &Hit, v: glm::Vec3, u: glm::Vec2) -> glm::Vec3
{
    let mut h = cos_hemisphere_sample(u);
    h = (hit.tangent * h.x) + (hit.bitangent * h.y) + (hit.normal * h.z);

    if glm::dot(v, hit.normal) < 0f32 // TODO: Duplicate dot product
    {
        h.z *= -1f32;
    }

    return h;
}

// Microfacet Isotropic
fn pdf_specular_isotropic(hit: &Hit, material: &Material, v: glm::Vec3, l: glm::Vec3) -> f32
{
    let h = glm::normalize(v + l); //TODO: Extract h

    let n_dot_h = glm::dot(hit.normal, h);
    let mut alpha2 = material.roughness * material.roughness;
    alpha2 *= alpha2;

    let cos2_theta = n_dot_h * n_dot_h;
    let denom = cos2_theta * (alpha2 - 1f32) + 1f32;

    if  denom == 0f32
    {
        return 0f32;
    }

    let pdf_distribution = alpha2 * n_dot_h / (PI * denom * denom);
    return pdf_distribution / (glm::dot(v, h) * 4f32);
}

// Sample Microfacet Isotropic
fn sample_specular_isotropic(hit: &Hit, material: &Material, v: glm::Vec3, u: glm::Vec2) -> glm::Vec3
{
    let phi = (TWO_PI) * u[0];
    let alpha = material.roughness * material.roughness;

    let tan_theta2 = alpha * alpha * u[1] / (1f32 - u[1]);
    let cos_theta = 1f32 / (1f32 + tan_theta2).sqrt();
    let sin_theta = (1f32 - cos_theta * cos_theta).max(0f32).sqrt();

    let h_local = spherical_direction(sin_theta, cos_theta, phi.sin(), phi.cos());

    let mut h = (hit.tangent * h_local.x) + (hit.bitangent * h_local.y) + (hit.normal * h_local.z);

    if !same_hemisphere(&hit, v, h) {
        h = h * -1f32;
    }

    return reflect(-v, h);
}

pub fn pdf(hit: &Hit, material: &Material, v: glm::Vec3, l: glm::Vec3) -> f32
{
    if same_hemisphere(&hit, v, l)
    {
        let pdf_diff = pdf_diffuse(&hit, l);
        let pdf_spec = pdf_specular_isotropic(&hit, material, v, l);

        return (pdf_diff + pdf_spec) / 2f32;
    }
    else
    {
        return 0f32;
    }
}

pub fn sample(hit: &Hit, material: &Material, v: glm::Vec3, rng: &mut RandGenerator) -> glm::Vec3
{
    let u = next_rand_v2(rng);
    let rnd = next_rand(rng);

    if rnd <= 0.5f32
    {
        return sample_diffuse(&hit, v, u);
    }
    else
    {
        return sample_specular_isotropic(&hit, &material, v, u);
    }
}

pub fn evaluate(material: &Material, n_dot_l: f32, n_dot_v: f32, n_dot_h: f32, l_dot_h: f32) -> glm::Vec3
{
    let diffuse = diffuse(&material);
    let specular = specular_isotropic(&material, n_dot_l, n_dot_v, n_dot_h, l_dot_h);

    let retval = diffuse
        * (1f32 - material.metallic)
        + specular;

    return retval * n_dot_l;
}

pub fn direct_lighting(scene: &SceneGraph, hit: &Hit, material: &Material, v: glm::Vec3) -> glm::Vec3
{
    //let l = glm::normalize(glm::vec3(0.5f32, 1f32, -0.9f32));
    let l = glm::normalize(glm::vec3(0f32, 1f32, 0f32));
    let h = glm::normalize(v + l);

    // ----> NOTE: ZERO HERE DISABLES LIGHTING <----
    let emission = /*light color*/ glm::vec3(1f32, 1f32, 1f32) * 2f32;

    let obstructed = scene.traverse_shadow(hit.pos, l, 1000f32);
    let shadow_mul = !obstructed as i32 as f32;

    let n_dot_l = glm::dot(hit.normal, l).abs();
    let n_dot_v = glm::dot(hit.normal, v).abs().max(MIN_N_DOT_V); // TODO: minimum ndotv?
    let n_dot_h = glm::dot(hit.normal, h).abs();
    let l_dot_h = glm::dot(l, h).abs();

    //let shadow_term = shadow_terminator_term_chiang2019(&hit, l);
    let f = evaluate(&material, n_dot_l, n_dot_v, n_dot_h, l_dot_h) * /*shadow_term **/ shadow_mul;
    let lighting = f * emission;

    return lighting;
}
