mod loader;
pub use loader::*;

use bevy_app::prelude::*;
use bevy_asset::AddAsset;
use bevy_render::mesh::Mesh;

/// Adds support for Obj file loading to Apps
#[derive(Default)]
pub struct ObjPlugin;

impl Plugin for ObjPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset_loader::<Mesh, ObjLoader>();
    }
}
