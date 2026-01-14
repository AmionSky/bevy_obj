use std::path::{Path, PathBuf};

use crate::{ObjSettings, util::MeshConverter};
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
    // #[error("Failed to load materials for {0}: {1}")]
    // MaterialError(AssetPath<'static>, #[source] tobj::LoadError),
}
/*
async fn load_obj_data<'a>(
    mut bytes: &'a [u8],
    load_context: &'a mut LoadContext<'_>,
) -> tobj::LoadResult {
    tobj::futures::load_obj_buf(&mut bytes, &tobj::GPU_LOAD_OPTIONS, async |p| {
        use tobj::LoadError::OpenFileFailed;
        // We don't use the MTL material as an asset, just load the bytes of it.
        // But we are unable to call ctx.finish() and feed the result back. (which is no new asset)
        // Is this allowed?
        let mut ctx = load_context.begin_labeled_asset();
        let path = p
            .to_str()
            .and_then(|p| resolve_path(&ctx, p))
            .ok_or(OpenFileFailed)?;
        ctx.read_asset_bytes(path)
            .await
            .map_or(Err(OpenFileFailed), |bytes| {
                tobj::load_mtl_buf(&mut bytes.as_slice())
            })
    })
    .await
}
*/

fn resolve_path<P: AsRef<Path>>(ctx: &LoadContext, path: P) -> AssetPath<'static> {
    if let Some(parent) = ctx.path().parent() {
        parent.path().join(path).into()
    } else {
        path.as_ref().to_path_buf().into()
    }
}

async fn load_materials(
    ctx: &mut LoadContext<'_>,
    materials: HashSet<Option<(PathBuf, String)>>,
) -> Result<HashMap<Option<(PathBuf, String)>, Handle<StandardMaterial>>, ObjError> {
    let mut material_handles = HashMap::new();
    let mut mtl_libs = HashMap::new();

    for (i, mat_key) in materials.into_iter().enumerate() {
        material_handles.insert(
            mat_key.clone(),
            if let Some((path, name)) = mat_key {
                let mtl_mat =
                    get_or_load_mtl(ctx, &mut mtl_libs, path.to_path_buf(), &name).await?;
                let mat = convert_material(ctx, mtl_mat);
                ctx.add_labeled_asset(format!("Material.{i}.{name}"), mat)
            } else {
                ctx.add_labeled_asset("Material.Default".to_string(), StandardMaterial::default())
            },
        );
    }

    Ok(material_handles)
}

async fn get_or_load_mtl<'m>(
    ctx: &mut LoadContext<'_>,
    mtl_libs: &'m mut HashMap<PathBuf, wobj::Mtl>,
    path: PathBuf,
    name: &str,
) -> Result<&'m wobj::Material, ObjError> {
    if !mtl_libs.contains_key(&path) {
        let asset_path = resolve_path(ctx, &path);
        let bytes = ctx.read_asset_bytes(asset_path).await?;
        let mtl = wobj::Mtl::parse(&bytes)?;
        mtl_libs.insert(path.clone(), mtl);
    }

    if let Some(mtl) = mtl_libs.get(&path)
        && let Some(material) = mtl.get(name)
    {
        Ok(material)
    } else {
        Err(ObjError::MatNotFound)
    }
}

fn load_texture<P: AsRef<Path>>(ctx: &mut LoadContext, texture: P) -> Option<Handle<Image>> {
    Some(ctx.load(resolve_path(ctx, texture)))
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

    if let Some(v) = &material.roughness {
        m.perceptual_roughness = *v;
    }
    if let Some(v) = &material.metallic {
        m.metallic = *v;
    }
    if let Some(v) = &material.cc_thickness {
        m.clearcoat = *v;
    }
    if let Some(v) = &material.cc_roughness {
        m.clearcoat_perceptual_roughness = *v;
    }
    if let Some(v) = &material.anisotropy {
        m.anisotropy_strength = *v;
    }
    if let Some(v) = &material.anisotropy_rotation {
        m.anisotropy_rotation = *v;
    }

    if let Some(map) = &material.diffuse_map {
        m.base_color_texture = load_texture(ctx, map.path())
    }
    if let Some(map) = material.normal_map.as_ref().or(material.bump_map.as_ref()) {
        m.normal_map_texture = load_texture(ctx, map.path())
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
        let material = if let Some(mat_lib) = obj_mesh.mtllib()
            && let Some(mat_name) = obj_mesh.material()
        {
            Some((mat_lib.to_path_buf(), mat_name.to_string()))
        } else {
            None
        };

        let (indicies, vertices) = obj_mesh.triangulate();

        materials.insert(material.clone());
        meshes.push((indicies, vertices, material));
    }

    let mat_handles = load_materials(ctx, materials).await?;

    let mut world = World::default();
    for (i, (indicies, verticies, mat_key)) in meshes.into_iter().enumerate() {
        let mesh_handle = ctx.add_labeled_asset(
            format!("Mesh.{i}"),
            MeshConverter::new(indicies, verticies).convert(settings),
        );

        let entity = (
            Mesh3d(mesh_handle),
            MeshMaterial3d(mat_handles.get(&mat_key).unwrap().clone()),
        );

        println!("Adding: {:#?}", entity);

        world.spawn(entity);
    }

    Ok(Scene::new(world))
}
