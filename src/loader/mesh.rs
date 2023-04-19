use anyhow::Result;
use bevy_asset::{LoadContext, LoadedAsset};
use bevy_render::{
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use thiserror::Error;

#[allow(clippy::derivable_impls)]
impl Default for super::ObjLoader {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Error, Debug)]
pub enum ObjError {
    #[error("Invalid OBJ file: {0}")]
    InvalidFile(#[from] tobj::LoadError),
}

pub(super) async fn load_obj<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), ObjError> {
    let mesh = load_obj_from_bytes(bytes)?;
    load_context.set_default_asset(LoadedAsset::new(mesh));
    Ok(())
}

pub fn load_obj_from_bytes(mut bytes: &[u8]) -> Result<Mesh, ObjError> {
    let options = tobj::GPU_LOAD_OPTIONS;
    let obj = tobj::load_obj_buf(&mut bytes, &options, |_| {
        Err(tobj::LoadError::GenericFailure)
    })?;

    let mut indices = Vec::new();
    let mut vertex_position = Vec::new();
    let mut vertex_normal = Vec::new();
    let mut vertex_texture = Vec::new();
    for model in obj.0 {
        let index_offset = vertex_position.len() as u32; // Offset of the indices
        indices.reserve(model.mesh.indices.len());
        vertex_position.reserve(model.mesh.positions.len() / 3);
        vertex_normal.reserve(model.mesh.normals.len() / 3);
        vertex_texture.reserve(model.mesh.texcoords.len() / 2);
        vertex_position.extend(
            model
                .mesh
                .positions
                .chunks_exact(3)
                .map(|v| [v[0], v[1], v[2]]),
        );
        vertex_normal.extend(
            model
                .mesh
                .normals
                .chunks_exact(3)
                .map(|n| [n[0], n[1], n[2]]),
        );
        vertex_texture.extend(
            model
                .mesh
                .texcoords
                .chunks_exact(2)
                .map(|t| [t[0], 1.0 - t[1]]),
        );
        indices.extend(model.mesh.indices.iter().map(|i| i + index_offset));
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_position);
    if !vertex_texture.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vertex_texture);
    }

    if !vertex_normal.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normal);
    } else {
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
    }

    Ok(mesh)
}
