#[cfg(feature = "mesh")]
pub mod mesh;
#[cfg(feature = "scene")]
pub mod scene;

mod util;

use bevy::app::{App, Plugin};
use bevy::asset::AssetApp;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const EXTENSIONS: &[&str; 2] = &["obj", "OBJ"];

/// Adds support for OBJ asset loading
#[derive(Default)]
pub struct ObjPlugin;

impl Plugin for ObjPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "mesh")]
        app.init_asset_loader::<mesh::ObjLoader>();
        #[cfg(feature = "scene")]
        app.init_asset_loader::<scene::ObjLoader>();
    }
}

/// OBJ asset loader settings
pub struct ObjSettings {
    /// Force compute the normals even if the mesh contains normals
    pub force_compute_normals: bool,
    /// Prefer flat normals over smooth normals when computing them
    pub prefer_flat_normals: bool,
    /// load options for asset loader
    pub load_options: tobj::LoadOptions,
}

impl Default for ObjSettings {
    fn default() -> Self {
        Self {
            force_compute_normals: Default::default(),
            prefer_flat_normals: Default::default(),
            #[cfg(feature = "mesh")]
            load_options: tobj::OFFLINE_RENDERING_LOAD_OPTIONS,
            #[cfg(feature = "scene")]
            load_options: tobj::GPU_LOAD_OPTIONS,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoadOptionsDef {
    #[cfg(feature = "merging")]
    pub merge_identical_points: bool,
    #[cfg(feature = "reordering")]
    pub reorder_data: bool,
    pub single_index: bool,
    pub triangulate: bool,
    pub ignore_points: bool,
    pub ignore_lines: bool,
}

impl From<LoadOptionsDef> for tobj::LoadOptions {
    fn from(def: LoadOptionsDef) -> tobj::LoadOptions {
        tobj::LoadOptions {
            #[cfg(feature = "merging")]
            merge_identical_points: def.merge_identical_points,
            #[cfg(feature = "reordering")]
            reorder_data: def.reorder_data,
            single_index: def.single_index,
            triangulate: def.triangulate,
            ignore_points: def.ignore_points,
            ignore_lines: def.ignore_lines,
        }
    }
}

impl From<tobj::LoadOptions> for LoadOptionsDef {
    fn from(lo: tobj::LoadOptions) -> LoadOptionsDef {
        LoadOptionsDef {
            #[cfg(feature = "merging")]
            merge_identical_points: lo.merge_identical_points,
            #[cfg(feature = "reordering")]
            reorder_data: lo.reorder_data,
            single_index: lo.single_index,
            triangulate: lo.triangulate,
            ignore_points: lo.ignore_points,
            ignore_lines: lo.ignore_lines,
        }
    }
}

/// Small helper struct used only for (de)serialization.
/// It mirrors ObjSettings but uses the serde-able LoadOptionsDef.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableObjSettings {
    pub force_compute_normals: bool,
    pub prefer_flat_normals: bool,
    pub load_options: LoadOptionsDef,
}

impl From<&ObjSettings> for SerializableObjSettings {
    fn from(s: &ObjSettings) -> SerializableObjSettings {
        SerializableObjSettings {
            force_compute_normals: s.force_compute_normals,
            prefer_flat_normals: s.prefer_flat_normals,
            load_options: LoadOptionsDef::from(s.load_options),
        }
    }
}

impl From<SerializableObjSettings> for ObjSettings {
    fn from(s: SerializableObjSettings) -> ObjSettings {
        ObjSettings {
            force_compute_normals: s.force_compute_normals,
            prefer_flat_normals: s.prefer_flat_normals,
            load_options: tobj::LoadOptions::from(s.load_options),
        }
    }
}

impl Serialize for ObjSettings {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializableObjSettings::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ObjSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ser: SerializableObjSettings = Deserialize::deserialize(deserializer)?;
        Ok(ObjSettings::from(ser))
    }
}
