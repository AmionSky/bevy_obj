#[cfg(feature = "scene")]
pub mod scene;
#[cfg(feature = "scene")]
pub use scene::*;

#[cfg(not(feature = "scene"))]
pub mod mesh;
#[cfg(not(feature = "scene"))]
pub use mesh::*;

use bevy_asset::{AssetLoader, BoxedFuture, LoadContext};

pub struct ObjLoader {
    #[cfg(feature = "scene")]
    supported_compressed_formats: bevy_render::texture::CompressedImageFormats,
}

impl AssetLoader for ObjLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            Ok(load_obj(
                bytes,
                load_context,
                #[cfg(feature = "scene")]
                self.supported_compressed_formats,
            )
            .await?)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["obj"]
    }
}
