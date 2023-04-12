use anyhow::Result;
use bevy_asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy_render::{
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use bevy_utils::BoxedFuture;
use thiserror::Error;

#[derive(Default)]
pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy_asset::LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move { Ok(load_obj(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["obj"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
pub enum ObjError {
    #[error("Invalid OBJ file: {0}")]
    TobjError(#[from] tobj::LoadError),
    #[error("Unexpected number of meshes, only 1 is supported")]
    WrongMeshNumber,
}

async fn load_obj<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), ObjError> {
    let mesh = load_obj_from_bytes(bytes)?;
    load_context.set_default_asset(LoadedAsset::new(mesh));
    Ok(())
}

fn load_mtl(_path: &std::path::Path) -> tobj::MTLLoadResult {
    //TODO(luca) Implement mtl loading
    Err(tobj::LoadError::OpenFileFailed)
}

pub fn load_obj_from_bytes(mut bytes: &[u8]) -> Result<Mesh, ObjError> {
    let options = tobj::GPU_LOAD_OPTIONS;
    match tobj::load_obj_buf(&mut bytes, &options, load_mtl) {
        Ok(obj) => {
            if obj.0.len() != 1 {
                return Err(ObjError::WrongMeshNumber);
            }
            let mesh = &obj.0[0].mesh;

            let vertex_position: Vec<[f32; 3]> = mesh
                .positions
                .chunks_exact(3)
                .map(|v| [v[0], v[1], v[2]])
                .collect();
            let vertex_normal: Vec<[f32; 3]> = mesh
                .normals
                .chunks_exact(3)
                .map(|n| [n[0], n[1], n[2]])
                .collect();
            let vertex_texture: Vec<[f32; 2]> = mesh
                .texcoords
                .chunks_exact(2)
                .map(|t| [t[0], 1.0 - t[1]])
                .collect();

            let mut bevy_mesh = Mesh::new(PrimitiveTopology::TriangleList);

            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_position);
            if !vertex_texture.is_empty() {
                bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vertex_texture);
            }

            if !vertex_normal.is_empty() {
                bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normal);
            } else {
                bevy_mesh.compute_flat_normals();
            }

            bevy_mesh.set_indices(Some(Indices::U32(mesh.indices.clone())));

            Ok(bevy_mesh)
        }
        Err(err) => Err(ObjError::TobjError(err)),
    }
}
