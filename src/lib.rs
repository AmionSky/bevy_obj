#[cfg(feature = "mesh")]
pub mod mesh;
#[cfg(feature = "scene")]
pub mod scene;

use bevy_app::{App, Plugin};
use bevy_asset::AssetApp;

const EXTENSIONS: &[&str; 1] = &["obj"];

/// Adds support for OBJ asset loading
#[derive(Default)]
pub struct ObjPlugin;

impl Plugin for ObjPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "mesh")]
        app.preregister_asset_loader::<mesh::ObjLoader>(EXTENSIONS);
        #[cfg(feature = "scene")]
        app.preregister_asset_loader::<scene::ObjLoader>(EXTENSIONS);
    }

    fn finish(&self, app: &mut App) {
        #[cfg(feature = "mesh")]
        app.register_asset_loader(mesh::ObjLoader);
        #[cfg(feature = "scene")]
        app.register_asset_loader(scene::ObjLoader);
    }
}
