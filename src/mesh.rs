use crate::{ObjSettings, util::MeshConverter};
use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::mesh::Mesh;
use bevy::tasks::ConditionalSendFuture;

#[derive(Default)]
pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    type Error = ObjError;
    type Settings = ObjSettings;
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

pub fn load_obj_as_mesh(mut bytes: &[u8], settings: &ObjSettings) -> Result<Mesh, ObjError> {
    let obj = tobj::load_obj_buf(&mut bytes, &tobj::GPU_LOAD_OPTIONS, |_| {
        Err(tobj::LoadError::GenericFailure)
    })?;

    Ok(MeshConverter::from(obj.0).convert(settings))
}
