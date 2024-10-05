use crate::{ObjSettings, convert_uv, convert_vec3};
use bevy_asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};
use bevy_render::{
    mesh::{Indices, Mesh},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};
use bevy_utils::ConditionalSendFuture;

pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    type Error = ObjError;
    type Settings = ObjSettings;
    type Asset = Mesh;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            load_obj_as_mesh(&bytes, settings)
        })
    }

    fn extensions(&self) -> &[&str] {
        crate::EXTENSIONS
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ObjError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid OBJ file: {0}")]
    InvalidFile(#[from] tobj::LoadError),
}

pub fn load_obj_as_mesh(mut bytes: &[u8], settings: &ObjSettings) -> Result<Mesh, ObjError> {
    let options = tobj::GPU_LOAD_OPTIONS;
    let obj = tobj::load_obj_buf(&mut bytes, &options, |_| {
        Err(tobj::LoadError::GenericFailure)
    })?;

    let mut indices = Vec::new();
    let mut vertex_position = Vec::new();
    let mut vertex_normal = Vec::new();
    let mut vertex_texture = Vec::new();

    for model in obj.0 {
        // Get the offset of the indices
        let index_offset = vertex_position.len() as u32;

        // Reserve the exact space needed in the vector
        indices.reserve(model.mesh.indices.len());
        vertex_position.reserve(model.mesh.positions.len() / 3);
        vertex_normal.reserve(model.mesh.normals.len() / 3);
        vertex_texture.reserve(model.mesh.texcoords.len() / 2);

        // Extend the vector
        indices.extend(model.mesh.indices.into_iter().map(|i| i + index_offset));
        vertex_position.extend(convert_vec3(model.mesh.positions));
        vertex_normal.extend(convert_vec3(model.mesh.normals));
        vertex_texture.extend(convert_uv(model.mesh.texcoords));
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_indices(Indices::U32(indices));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_position);

    if !vertex_texture.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vertex_texture);
    }

    if !vertex_normal.is_empty() && !settings.force_compute_normals {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normal);
    } else if settings.prefer_flat_normals {
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
    } else {
        mesh.compute_normals();
    }

    Ok(mesh)
}
