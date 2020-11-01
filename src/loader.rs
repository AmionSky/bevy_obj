use anyhow::Result;
use bevy_asset::AssetLoader;
use bevy_render::{
    mesh::{Mesh, VertexAttribute},
    pipeline::PrimitiveTopology,
};
use std::path::Path;
use thiserror::Error;

#[derive(Default)]
pub struct ObjLoader;

impl AssetLoader<Mesh> for ObjLoader {
    fn from_bytes(&self, _asset_path: &Path, bytes: Vec<u8>) -> Result<Mesh> {
        let mesh = load_obj(bytes)?;
        Ok(mesh)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["obj"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
pub enum ObjError {
    #[error("Invalid OBJ file.")]
    Gltf(#[from] obj::ObjError),
}

fn load_obj(bytes: Vec<u8>) -> Result<Mesh, ObjError> {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    load_obj_pnt(obj::load_obj(bytes.as_slice())?, &mut mesh);
    Ok(mesh)
}

fn load_obj_pnt(obj: obj::Obj<obj::TexturedVertex>, mesh: &mut Mesh) {
    let uvs = VertexAttributeValues::Float3(obj.vertices
                .iter()
                // Flip UV for correct values
                .map(|v| [v.texture[0], 1.0 - v.texture[1], v.texture[2]])
                .collect());
    let positions = VertexAttributeValues::Float3(obj.vertices.iter().map(|v| v.position).collect());
    let normals = VertexAttributeValues::Float3(obj.vertices.iter().map(|v| v.normal).collect());

    mesh.attributes.insert(Cow::Borrowed(Mesh::ATTRIBUTE_POSITION), positions);
    mesh.attributes.insert(Cow::Borrowed(Mesh::ATTRIBUTE_NORMAL), normals);
    mesh.attributes.insert(Cow::Borrowed(Mesh::ATTRIBUTE_UV_0), uvs);

    set_mesh_indices(mesh, obj);
}

fn set_mesh_indices<T>(mesh: &mut Mesh, obj: obj::Obj<T>) {
    mesh.indices = Some(obj.indices.iter().map(|i| *i as u32).collect());
}
