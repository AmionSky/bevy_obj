#[cfg(feature = "mesh")]
pub mod mesh;
#[cfg(feature = "scene")]
pub mod scene;

mod util;

use std::marker::PhantomData;

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
        app.init_asset_loader::<mesh::MeshObjLoader>();
        #[cfg(feature = "scene")]
        app.init_asset_loader::<scene::SceneObjLoader>();
    }
}

/// OBJ asset loader settings
pub struct ObjSettings<Loader> {
    loader: PhantomData<Loader>,
    /// Force compute the normals even if the mesh contains normals
    pub force_compute_normals: bool,
    /// Prefer flat normals over smooth normals when computing them
    pub prefer_flat_normals: bool,
    /// load options for asset loader
    pub load_options: tobj::LoadOptions,
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

impl<Loader> From<&ObjSettings<Loader>> for SerializableObjSettings {
    fn from(s: &ObjSettings<Loader>) -> SerializableObjSettings {
        SerializableObjSettings {
            force_compute_normals: s.force_compute_normals,
            prefer_flat_normals: s.prefer_flat_normals,
            load_options: LoadOptionsDef::from(s.load_options),
        }
    }
}

impl<Loader> From<SerializableObjSettings> for ObjSettings<Loader> {
    fn from(s: SerializableObjSettings) -> ObjSettings<Loader> {
        ObjSettings::<Loader> {
            loader: PhantomData,
            force_compute_normals: s.force_compute_normals,
            prefer_flat_normals: s.prefer_flat_normals,
            load_options: tobj::LoadOptions::from(s.load_options),
        }
    }
}

impl<Loader> Serialize for ObjSettings<Loader> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializableObjSettings::from(self).serialize(serializer)
    }
}

impl<'de, Loader> Deserialize<'de> for ObjSettings<Loader> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ser: SerializableObjSettings = Deserialize::deserialize(deserializer)?;
        Ok(ObjSettings::from(ser))
    }
}
