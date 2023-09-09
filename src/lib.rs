mod loader;
pub use loader::*;

use bevy_app::prelude::*;
use bevy_asset::AssetApp;

const EXTENSIONS: &[&str; 1] = &["obj"];

/// Adds support for Obj file loading to Apps
#[derive(Default)]
pub struct ObjPlugin;

impl Plugin for ObjPlugin {
    fn build(&self, app: &mut App) {
        app.preregister_asset_loader::<ObjLoader>(EXTENSIONS);
    }

    fn finish(&self, app: &mut App) {
        app.register_asset_loader(ObjLoader);
    }
}
