#[cfg(feature = "scene")]
pub mod scene;
#[cfg(feature = "scene")]
pub use scene::*;

#[cfg(not(feature = "scene"))]
pub mod mesh;
#[cfg(not(feature = "scene"))]
pub use mesh::*;

use anyhow::Result;
use bevy_asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};
use bevy_utils::BoxedFuture;

pub struct ObjLoader {
    #[cfg(feature = "scene")]
    pub supported_compressed_formats: bevy_render::texture::CompressedImageFormats,
}

impl AssetLoader for ObjLoader {
    type Settings = ();
    #[cfg(not(feature = "scene"))]
    type Asset = bevy_render::mesh::Mesh;
    #[cfg(feature = "scene")]
    type Asset = bevy_scene::Scene;
    

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            Ok(load_obj(self, &bytes, load_context).await?)
        })
    }

    fn extensions(&self) -> &[&str] {
        super::EXTENSIONS
    }
}
