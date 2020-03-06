extern crate assimp;
extern crate itertools;

use itertools::izip;
use crate::resource_manager::*;
use assimp::import::Importer;

pub type ModelManager = ResourceManager<Model, ModelLoader>;

#[derive(Debug)]
pub struct Model
{
    pub meshes: Vec<Mesh>,
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex
{
    pub pos: glm::Vec3,
    pub normal: glm::Vec3,
    pub tangent: glm::Vec3,
    pub bitangent: glm::Vec3,
    pub uv: glm::Vec2,
}

#[derive(Debug)]
pub struct Mesh
{
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Default for Model
{
    fn default() -> Self
    {
        Model
        {
            meshes: Vec::new(),
        }
    }
}

impl Default for Mesh
{
    fn default() -> Self
    {
        Mesh
        {
            vertices: Vec::new(),
            indices: Vec::new()
        }
    }
}

impl Default for Vertex
{
    fn default() -> Self
    {
        Vertex
        {
            pos: glm::vec3(0f32, 0f32, 0f32),
            normal: glm::vec3(1f32, 0f32, 0f32),
            tangent: glm::vec3(1f32, 0f32, 0f32),
            bitangent: glm::vec3(1f32, 0f32, 0f32),
            uv: glm::vec2(0f32, 0f32),
        }
    }
}

pub struct ModelLoader
{
}

impl ResourceLoader<Model> for ModelLoader {
    type Args = str;
    fn load(&self, path: &str) -> Result<Model, String>
    {
        let mut model = Model::default();

        let mut importer = Importer::new();
        importer.triangulate(true);
        importer.calc_tangent_space(|x| x.enable = true);
        importer.pre_transform_vertices(|x| {
            x.enable = true;
            x.normalize = false
        });
        importer.join_identical_vertices(true);

        let scene = importer.read_file(path)?;
        for ai_mesh in scene.mesh_iter()
        {
            let mut mesh = Mesh::default();

            for (v, n, t, b, u) in izip!(ai_mesh.vertex_iter(), ai_mesh.normal_iter(), ai_mesh.tangent_iter(), ai_mesh.bitangent_iter(), ai_mesh.texture_coords_iter(0))
            {
                mesh.vertices.push(Vertex
                {
                    pos: glm::vec3(v.x, v.y, v.z),
                    normal: glm::vec3(n.x, n.y, n.z),
                    tangent: glm::vec3(t.x, t.y, t.z),
                    bitangent: glm::vec3(b.x, b.y, b.z),
                    uv: glm::vec2(u.x, u.y),
                });
            };

            for face in ai_mesh.face_iter()
            {
                mesh.indices.push(face[0]);
                mesh.indices.push(face[1]);
                mesh.indices.push(face[2]);
            }

            model.meshes.push(mesh);
        }

        Ok(model)
    }
}