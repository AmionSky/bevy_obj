use std::path::{Path, PathBuf};

use crate::ObjSettings;
use bevy::asset::AssetPath;
use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevy::tasks::ConditionalSendFuture;

#[derive(Default, TypePath)]
pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    type Error = ObjError;
    type Settings = ObjSettings;
    type Asset = Scene;

    fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            load_obj_as_scene(&bytes, load_context, settings).await
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
    ParseError(#[from] wobj::WobjError),
    #[error("Asset read error: {0}")]
    AssetError(#[from] bevy::asset::ReadAssetBytesError),
    #[error("Material not found")]
    MatNotFound,
    #[error("Invalid mesh: {0}")]
    InvalidMesh(&'static str),
}

fn resolve_path<P: AsRef<Path>>(ctx: &LoadContext, path: P) -> AssetPath<'static> {
    if let Some(parent) = ctx.path().parent() {
        parent.path().join(path).into()
    } else {
        path.as_ref().to_path_buf().into()
    }
}

struct MtlCache {
    cache: HashMap<PathBuf, wobj::Mtl>,
}

impl MtlCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::with_capacity(1),
        }
    }

    pub async fn load(
        &mut self,
        ctx: &mut LoadContext<'_>,
        path: &PathBuf,
        name: &str,
    ) -> Result<&wobj::Material, ObjError> {
        if !self.cache.contains_key(path) {
            let asset_path = resolve_path(ctx, path);
            let bytes = ctx.read_asset_bytes(asset_path).await?;
            let mtl = wobj::Mtl::parse(&bytes)?;
            self.cache.insert(path.clone(), mtl);
        }

        self.cache
            .get(path)
            .and_then(|mtl| mtl.get(name))
            .ok_or(ObjError::MatNotFound)
    }
}

async fn load_materials(
    ctx: &mut LoadContext<'_>,
    materials: HashSet<Option<(PathBuf, String)>>,
) -> Result<HashMap<Option<(PathBuf, String)>, Handle<StandardMaterial>>, ObjError> {
    let mut handles = HashMap::new();
    let mut mtls = MtlCache::new();

    fn default_material(ctx: &mut LoadContext<'_>) -> Handle<StandardMaterial> {
        const DEFAULT_MATERIAL_LABEL: &str = "Material.Default";
        if ctx.has_labeled_asset(DEFAULT_MATERIAL_LABEL) {
            ctx.get_label_handle(DEFAULT_MATERIAL_LABEL)
        } else {
            ctx.add_labeled_asset(
                DEFAULT_MATERIAL_LABEL.to_string(),
                StandardMaterial::default(),
            )
        }
    }

    for (i, mat_key) in materials.into_iter().enumerate() {
        let handle = if let Some((path, name)) = &mat_key {
            match mtls.load(ctx, path, name).await {
                Ok(mtl_mat) => {
                    let material = convert_material(ctx, mtl_mat);
                    let label = format!("Material.{i}.{name}");
                    ctx.add_labeled_asset(label, material)
                }
                Err(error) => {
                    // TODO: properly log error
                    eprintln!("Failed to load material: {error}");
                    default_material(ctx)
                }
            }
        } else {
            default_material(ctx)
        };

        handles.insert(mat_key, handle);
    }

    Ok(handles)
}

fn load_texture(ctx: &mut LoadContext, path: &PathBuf) -> Handle<Image> {
    ctx.load(resolve_path(ctx, path))
}

fn convert_material(ctx: &mut LoadContext<'_>, material: &wobj::Material) -> StandardMaterial {
    let mut m = StandardMaterial::default();
    if let Some(v) = &material.diffuse {
        match *v {
            wobj::ColorValue::RGB(r, g, b) => m.base_color = Color::srgb(r, g, b),
            wobj::ColorValue::XYZ(x, y, z) => m.base_color = Color::xyz(x, y, z),
            _ => (),
        }
    }
    if let Some(v) = &material.specular {
        match *v {
            wobj::ColorValue::RGB(r, g, b) => m.specular_tint = Color::srgb(r, g, b),
            wobj::ColorValue::XYZ(x, y, z) => m.specular_tint = Color::xyz(x, y, z),
            _ => (),
        }
    }
    if let Some(v) = &material.emissive {
        match *v {
            wobj::ColorValue::RGB(r, g, b) => m.emissive = LinearRgba::rgb(r, g, b),
            wobj::ColorValue::XYZ(x, y, z) => m.emissive = Color::xyz(x, y, z).into(),
            _ => (),
        }
    }

    fn apply_f32(input: Option<f32>, target: &mut f32) {
        *target = input.unwrap_or(*target)
    }

    apply_f32(material.roughness, &mut m.perceptual_roughness);
    apply_f32(material.metallic, &mut m.metallic);
    apply_f32(material.cc_thickness, &mut m.clearcoat);
    apply_f32(material.cc_roughness, &mut m.clearcoat_perceptual_roughness);
    apply_f32(material.anisotropy, &mut m.anisotropy_strength);
    apply_f32(material.anisotropy_rotation, &mut m.anisotropy_rotation);

    if let Some(map) = &material.diffuse_map {
        m.base_color_texture = Some(load_texture(ctx, map.path()))
    }

    // Treat both normal and bump map as normal map texture
    if let Some(map) = material.normal_map.as_ref().or(material.bump_map.as_ref()) {
        m.normal_map_texture = Some(load_texture(ctx, map.path()))
    }

    // Enable alpha blend if the material has a dissolve map
    if material.dissolve_map.is_some() {
        m.alpha_mode = AlphaMode::Blend
    }

    m
}

async fn load_obj_as_scene<'a>(
    bytes: &'a [u8],
    ctx: &'a mut LoadContext<'_>,
    settings: &'a ObjSettings,
) -> Result<Scene, ObjError> {
    let obj = wobj::Obj::parse(bytes)?;

    let mut materials = HashSet::new();
    let mut meshes = Vec::new();

    for obj_mesh in obj.meshes() {
        let material = obj_mesh
            .mtllib()
            .map(PathBuf::from)
            .zip(obj_mesh.material().map(String::from));
        materials.insert(material.clone());

        let (indicies, vertices) = obj_mesh.triangulate().map_err(ObjError::InvalidMesh)?;
        meshes.push((indicies, vertices, material));
    }

    let mat_handles = load_materials(ctx, materials).await?;

    let mut world = World::default();
    for (i, (indicies, verticies, mat_key)) in meshes.into_iter().enumerate() {
        println!(
            "Adding mesh {i} with material {:?}",
            mat_key.as_ref().map(|(_, n)| n)
        );

        let mesh_handle = ctx.add_labeled_asset(
            format!("Mesh.{i}"),
            crate::util::to_bevy_mesh(indicies, verticies, settings),
        );

        let entity = (
            Mesh3d(mesh_handle),
            MeshMaterial3d(mat_handles.get(&mat_key).unwrap().clone()),
        );

        world.spawn(entity);
    }

    Ok(Scene::new(world))
}
