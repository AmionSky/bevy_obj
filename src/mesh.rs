use crate::ObjSettings;
use crate::util::MeshConverter;
use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::mesh::Mesh;
use bevy::tasks::ConditionalSendFuture;
use std::marker::PhantomData;

#[derive(Default)]
pub struct MeshObjLoader;

type MeshObjSettings = ObjSettings<MeshObjLoader>;

impl Default for MeshObjSettings {
    fn default() -> Self {
        Self {
            loader: PhantomData,
            force_compute_normals: Default::default(),
            prefer_flat_normals: Default::default(),
            load_options: tobj::OFFLINE_RENDERING_LOAD_OPTIONS,
        }
    }
}

impl AssetLoader for MeshObjLoader {
    type Error = ObjError;
    type Settings = MeshObjSettings;
    type Asset = Mesh;

    fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        _: &mut LoadContext,
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

pub fn load_obj_as_mesh(mut bytes: &[u8], settings: &MeshObjSettings) -> Result<Mesh, ObjError> {
    let obj = tobj::load_obj_buf(&mut bytes, &settings.load_options, |_| {
        Err(tobj::LoadError::GenericFailure)
    })?;

    Ok(MeshConverter::from(obj.0).convert(settings))
}
