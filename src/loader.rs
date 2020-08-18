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
    mesh.attributes.push(VertexAttribute::position(
        obj.vertices.iter().map(|v| v.position).collect(),
    ));
    mesh.attributes.push(VertexAttribute::normal(
        obj.vertices.iter().map(|v| v.normal).collect(),
    ));
    mesh.attributes.push(VertexAttribute::uv(
        obj.vertices
            .iter()
            // Flip UV for correct values
            .map(|v| [v.texture[0], 1.0 - v.texture[1]])
            .collect(),
    ));

    set_mesh_indices(mesh, obj);
}

fn set_mesh_indices<T>(mesh: &mut Mesh, obj: obj::Obj<T>) {
    mesh.indices = Some(obj.indices.iter().map(|i| *i as u32).collect());
}
