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
    #[error("Unknown vertex format.")]
    UnknownVertexFormat,
}

fn load_obj(bytes: Vec<u8>) -> Result<Mesh, ObjError> {
    let raw = obj::raw::parse_obj(bytes.as_slice())?;

    // Get the most complete vertex representation
    //  3 => Position, Normal, Texture
    //  2 => Position, Normal
    //  1 => Position
    let mut pnt = 3;
    for polygon in &raw.polygons {
        use obj::raw::object::Polygon;
        match polygon {
            Polygon::P(_) => pnt = std::cmp::min(pnt, 1),
            Polygon::PN(_) => pnt = std::cmp::min(pnt, 2),
            _ => {}
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
    match pnt {
        1 => load_obj_p(obj::Obj::new(raw)?, &mut mesh),
        2 => load_obj_pn(obj::Obj::new(raw)?, &mut mesh),
        3 => load_obj_pnt(obj::Obj::new(raw)?, &mut mesh),
        _ => return Err(ObjError::UnknownVertexFormat),
    }

    Ok(mesh)
}

fn load_obj_p(obj: obj::Obj<obj::Position>, mesh: &mut Mesh) {
    mesh.attributes.push(VertexAttribute::position(
        obj.vertices.iter().map(|v| v.position).collect(),
    ));

    set_mesh_indices(mesh, obj);
}

fn load_obj_pn(obj: obj::Obj<obj::Vertex>, mesh: &mut Mesh) {
    mesh.attributes.push(VertexAttribute::position(
        obj.vertices.iter().map(|v| v.position).collect(),
    ));
    mesh.attributes.push(VertexAttribute::normal(
        obj.vertices.iter().map(|v| v.normal).collect(),
    ));

    set_mesh_indices(mesh, obj);
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
            .map(|v| [v.texture[0], v.texture[1]])
            .collect(),
    ));

    set_mesh_indices(mesh, obj);
}

fn set_mesh_indices<T>(mesh: &mut Mesh, obj: obj::Obj<T>) {
    mesh.indices = Some(obj.indices.iter().map(|i| *i as u32).collect());
}
