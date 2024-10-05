use crate::{convert_uv, convert_vec3, ObjSettings};
use bevy_asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt, Handle, LoadContext};
use bevy_color::Color;
use bevy_ecs::world::World;
use bevy_pbr::{PbrBundle, StandardMaterial};
use bevy_render::{
    mesh::{Indices, Mesh},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
    texture::Image,
};
use bevy_scene::Scene;
use bevy_utils::ConditionalSendFuture;
use std::path::PathBuf;

pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    type Error = ObjError;
    type Settings = ObjSettings;
    type Asset = Scene;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
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
    TobjError(#[from] tobj::LoadError),
    #[error("Failed to load materials for {0}: {1}")]
    MaterialError(PathBuf, #[source] tobj::LoadError),
}

async fn load_obj_data<'a, 'b>(
    mut bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> tobj::LoadResult {
    let options = tobj::GPU_LOAD_OPTIONS;
    tobj::load_obj_buf_async(&mut bytes, &options, |p| async {
        use tobj::LoadError::OpenFileFailed;
        // We don't use the MTL material as an asset, just load the bytes of it.
        // But we are unable to call ctx.finish() and feed the result back. (which is no new asset)
        // Is this allowed?
        let mut ctx = load_context.begin_labeled_asset();
        ctx.read_asset_bytes(resolve_path(&ctx, p).ok_or(OpenFileFailed)?)
            .await
            .map_or(Err(OpenFileFailed), |bytes| {
                tobj::load_mtl_buf(&mut bytes.as_slice())
            })
    })
    .await
}

fn load_texture(texture: &String, ctx: &mut LoadContext) -> Option<Handle<Image>> {
    Some(ctx.load(resolve_path(ctx, texture)?))
}

fn resolve_path<P: AsRef<str>>(ctx: &LoadContext, path: P) -> Option<AssetPath<'static>> {
    ctx.asset_path().parent()?.resolve(path.as_ref()).ok()
}

async fn load_obj_as_scene<'a, 'b>(
    bytes: &'a [u8],
    ctx: &'a mut LoadContext<'b>,
    settings: &'a ObjSettings,
) -> Result<Scene, ObjError> {
    let (models, materials) = load_obj_data(bytes, ctx).await?;
    let materials = materials.map_err(|err| {
        let obj_path = ctx.path().to_path_buf();
        ObjError::MaterialError(obj_path, err)
    })?;

    let mut mat_handles = Vec::with_capacity(materials.len());
    for (mat_idx, mat) in materials.into_iter().enumerate() {
        let mut material = StandardMaterial {
            base_color_texture: mat.diffuse_texture.and_then(|t| load_texture(&t, ctx)),
            normal_map_texture: mat.normal_texture.and_then(|t| load_texture(&t, ctx)),
            ..Default::default()
        };
        if let Some(color) = mat.diffuse {
            material.base_color = Color::srgb(color[0], color[1], color[2]);
        }
        mat_handles.push(ctx.add_labeled_asset(format!("Material{mat_idx}"), material));
    }

    let mut world = World::default();
    for (model_idx, model) in models.into_iter().enumerate() {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        mesh.insert_indices(Indices::U32(model.mesh.indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, convert_vec3(model.mesh.positions));

        if !model.mesh.texcoords.is_empty() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, convert_uv(model.mesh.texcoords));
        }

        if !model.mesh.normals.is_empty() && !settings.force_compute_normals {
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, convert_vec3(model.mesh.normals));
        } else if settings.prefer_flat_normals {
            mesh.duplicate_vertices();
            mesh.compute_flat_normals();
        } else {
            mesh.compute_normals();
        }

        let mesh_handle = ctx.add_labeled_asset(format!("Mesh{model_idx}"), mesh);

        let pbr_bundle = PbrBundle {
            mesh: mesh_handle,
            material: model
                .mesh
                .material_id
                .map(|id| mat_handles[id].clone())
                .unwrap_or_default(),
            ..Default::default()
        };

        world.spawn(pbr_bundle);
    }

    Ok(Scene::new(world))
}
