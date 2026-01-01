use crate::{ObjSettings, util::MeshConverter};
use bevy::asset::{AssetLoader, AssetPath, LoadContext, io::Reader};
use bevy::prelude::*;
use bevy::tasks::ConditionalSendFuture;

#[derive(Default)]
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
    TobjError(#[from] tobj::LoadError),
    #[error("Failed to load materials for {0}: {1}")]
    MaterialError(std::path::PathBuf, #[source] tobj::LoadError),
}

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

fn load_texture(texture: &String, ctx: &mut LoadContext) -> Option<Handle<Image>> {
    // Parse MTL texture options and extract the filename
    // MTL format allows options before the filename (e.g., "-bm 1.0 texture.png")
    // Strategy: Skip all tokens starting with '-' and their associated values (numbers or on/off)
    // The filename is everything that remains after all options are consumed
    let mut tokens = texture.split_whitespace().peekable();
    
    while let Some(token) = tokens.peek() {
        if token.starts_with('-') {
            tokens.next(); // consume the option flag
            
            // Skip any following tokens that look like option values
            // (numbers, "on", "off", or single characters for -imfchan)
            while let Some(next_token) = tokens.peek() {
                if next_token.starts_with('-') {
                    // Next option found, stop consuming values
                    break;
                }
                
                // Check if this looks like an option value
                let is_number = next_token.parse::<f32>().is_ok();
                let is_boolean = *next_token == "on" || *next_token == "off";
                let is_single_char = next_token.len() == 1;
                
                if is_number || is_boolean || is_single_char {
                    tokens.next(); // consume the value
                } else {
                    // Doesn't look like an option value, must be start of filename
                    break;
                }
            }
        } else {
            // Not an option, must be the filename
            break;
        }
    }
    
    // Remaining tokens form the filename (which may contain spaces)
    let filename: String = tokens.collect::<Vec<_>>().join(" ");
    let final_texture = if filename.is_empty() { texture.as_str() } else { &filename };
    
    Some(ctx.load(resolve_path(ctx, final_texture)?))
}

fn resolve_path<P: AsRef<str>>(ctx: &LoadContext, path: P) -> Option<AssetPath<'static>> {
    ctx.asset_path().parent()?.resolve(path.as_ref()).ok()
}

async fn load_obj_as_scene<'a>(
    bytes: &'a [u8],
    ctx: &'a mut LoadContext<'_>,
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
        let material_id = model.mesh.material_id;
        let mesh_handle = ctx.add_labeled_asset(
            format!("Mesh{model_idx}"),
            MeshConverter::from(model).convert(settings),
        );

        let entity = (
            Mesh3d(mesh_handle),
            MeshMaterial3d(
                material_id
                    .map(|id| mat_handles[id].clone())
                    .unwrap_or_default(),
            ),
        );

        world.spawn(entity);
    }

    Ok(Scene::new(world))
}
