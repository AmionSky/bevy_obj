#[cfg(feature = "mesh")]
pub mod mesh;
#[cfg(feature = "scene")]
pub mod scene;

use bevy_app::{App, Plugin};
use bevy_asset::AssetApp;
use serde::{Serialize,Deserialize};

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

fn convert_vec3(vec: Vec<f32>) -> Vec<[f32; 3]> {
    vec.chunks_exact(3).map(|v| [v[0], v[1], v[2]]).collect()
}

fn convert_uv(uv: Vec<f32>) -> Vec<[f32; 2]> {
    uv.chunks_exact(2).map(|t| [t[0], 1.0 - t[1]]).collect()
}

/// OBJ asset loader settings
#[derive(Default, Serialize, Deserialize)]
pub struct ObjSettings {
    /// Force compute the normals even if the mesh contains normals
    pub force_compute_normals: bool,
    /// Prefer flat normals over smooth normals when computing them
    pub prefer_flat_normals: bool,
}