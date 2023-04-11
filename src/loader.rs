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
    //Gltf(#[from] obj::ObjError),
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

fn load_mtl(path: &std::path::Path) -> tobj::MTLLoadResult {
    Err(tobj::LoadError::OpenFileFailed)
}

pub fn load_obj_from_bytes(mut bytes: &[u8]) -> Result<Mesh, ObjError> {
    // TODO(luca) Consider removing single_index that comes with GPU load options
    // for memory efficiency
    let options = tobj::GPU_LOAD_OPTIONS;
    match tobj::load_obj_buf(&mut bytes, &options, load_mtl) {
        Ok(obj) => {
            if obj.0.len() != 1 {
                return Err(ObjError::WrongMeshNumber);
            }
            let mesh = &obj.0[0].mesh;

            let mut vertex_position = Vec::with_capacity(mesh.indices.len());
            let mut vertex_normal = Vec::with_capacity(mesh.indices.len());
            let mut vertex_texture = Vec::with_capacity(mesh.indices.len());

            for idx in &mesh.indices {
                let idx = *idx as usize;
                let pos = [
                    mesh.positions[3 * idx],
                    mesh.positions[3 * idx + 1],
                    mesh.positions[3 * idx + 2],
                ];
                vertex_position.push(pos);
                if !mesh.normals.is_empty() {
                    let norm = [
                        mesh.normals[3 * idx],
                        mesh.normals[3 * idx + 1],
                        mesh.normals[3 * idx + 2],
                    ];
                    vertex_normal.push(norm);
                }
                if !mesh.texcoords.is_empty() {
                    let texcoord = [mesh.texcoords[2 * idx], 1.0 - mesh.texcoords[2 * idx + 1]];
                    vertex_texture.push(texcoord);
                }
            }
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

            let indices = (0..mesh.indices.len()).map(|x| x as u32).collect();
            bevy_mesh.set_indices(Some(Indices::U32(indices)));

            Ok(bevy_mesh)
        }
        Err(err) => Err(ObjError::TobjError(err)),
    }
}
