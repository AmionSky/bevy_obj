use anyhow::Result;
use bevy_asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy_render::{
    mesh::{Indices, Mesh, VertexAttributeValues},
    pipeline::PrimitiveTopology,
};
use bevy_utils::BoxedFuture;
use std::borrow::Cow;
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
    #[error("Invalid OBJ file.")]
    Gltf(#[from] obj::ObjError),
}

async fn load_obj<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), ObjError> {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    load_obj_pnt(obj::load_obj(bytes)?, &mut mesh);
    load_context.set_default_asset(LoadedAsset::new(mesh));
    Ok(())
}

fn load_obj_pnt(obj: obj::Obj<obj::TexturedVertex>, mesh: &mut Mesh) {
    let uvs = VertexAttributeValues::Float3(
        obj.vertices
            .iter()
            // Flip UV for correct values
            .map(|v| [v.texture[0], 1.0 - v.texture[1], v.texture[2]])
            .collect(),
    );
    let positions =
        VertexAttributeValues::Float3(obj.vertices.iter().map(|v| v.position).collect());
    let normals = VertexAttributeValues::Float3(obj.vertices.iter().map(|v| v.normal).collect());

    mesh.attributes
        .insert(Cow::Borrowed(Mesh::ATTRIBUTE_POSITION), positions);
    mesh.attributes
        .insert(Cow::Borrowed(Mesh::ATTRIBUTE_NORMAL), normals);
    mesh.attributes
        .insert(Cow::Borrowed(Mesh::ATTRIBUTE_UV_0), uvs);

    set_mesh_indices(mesh, obj);
}

fn set_mesh_indices<T>(mesh: &mut Mesh, obj: obj::Obj<T>) {
    mesh.indices = Some(Indices::U32(
        obj.indices.iter().map(|i| *i as u32).collect(),
    ));
}
