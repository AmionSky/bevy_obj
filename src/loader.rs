use anyhow::Result;
use bevy_asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy_render::{
    mesh::{Indices, Mesh},
    render_resource::PrimitiveTopology,
};
use bevy_utils::{BoxedFuture, HashMap};
use obj::raw::{object::Polygon, RawObj};
use thiserror::Error;

#[derive(Default)]
pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy_asset::LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move { Ok(load_obj(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["obj"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
pub enum ObjError {
    #[error("Invalid OBJ file: {0}")]
    Gltf(#[from] obj::ObjError),
    #[error("Mesh is not triangulated.")]
    NonTriangulatedMesh,
}

async fn load_obj<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), ObjError> {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    load_obj_from_bytes(bytes, &mut mesh)?;
    load_context.set_default_asset(LoadedAsset::new(mesh));
    Ok(())
}

type VertexKey = (usize, usize, usize);

struct MeshIndices {
    indices: Vec<u32>,
    saved: HashMap<VertexKey, u32>,
    next: u32,
}

impl MeshIndices {
    pub fn new(capacity: usize) -> Self {
        Self {
            indices: Vec::with_capacity(capacity),
            saved: HashMap::with_capacity(capacity),
            next: 0,
        }
    }

    pub fn insert<F: FnOnce()>(&mut self, key: VertexKey, create_vertex: F) {
        // Check if the vertex is already saved
        match self.saved.get(&key) {
            Some(index) => self.indices.push(*index), // If saved, just use the existing index
            None => {
                // Save the index to both the indices and saved
                self.indices.push(self.next);
                self.saved.insert(key, self.next);
                // Increment next index
                self.next += 1;
                // Create a vertex externally
                create_vertex()
            }
        }
    }
}

impl From<MeshIndices> for Vec<u32> {
    fn from(val: MeshIndices) -> Self {
        val.indices
    }
}

pub fn load_obj_from_bytes(bytes: &[u8], mesh: &mut Mesh) -> Result<(), ObjError> {
    let raw = obj::raw::parse_obj(bytes)?;
    let vertcount = raw.polygons.len() * 3;

    let mut indices = MeshIndices::new(vertcount);

    let mut vertex_position = Vec::with_capacity(vertcount);
    let mut vertex_normal = Vec::with_capacity(vertcount);
    let mut vertex_texture = Vec::with_capacity(vertcount);

    for polygon in &raw.polygons {
        match polygon {
            Polygon::P(poly) if poly.len() == 3 => {
                let normal = calculate_normal(&raw, poly);

                for ipos in poly {
                    indices.insert((*ipos, 0, 0), || {
                        vertex_position.push(convert_position(&raw, *ipos));
                        vertex_normal.push(normal);
                    });
                }
            }
            Polygon::PT(poly) if poly.len() == 3 => {
                let triangle: Vec<usize> = poly.iter().map(|(ipos, _)| *ipos).collect();
                let normal = calculate_normal(&raw, &triangle);

                for (ipos, itex) in poly {
                    indices.insert((*ipos, 0, *itex), || {
                        vertex_position.push(convert_position(&raw, *ipos));
                        vertex_normal.push(normal);
                        vertex_texture.push(convert_texture(&raw, *itex));
                    });
                }
            }
            Polygon::PN(poly) if poly.len() == 3 => {
                for (ipos, inorm) in poly {
                    indices.insert((*ipos, *inorm, 0), || {
                        vertex_position.push(convert_position(&raw, *ipos));
                        vertex_normal.push(convert_normal(&raw, *inorm));
                    });
                }
            }
            Polygon::PTN(poly) if poly.len() == 3 => {
                for (ipos, itex, inorm) in poly {
                    indices.insert((*ipos, *inorm, *itex), || {
                        vertex_position.push(convert_position(&raw, *ipos));
                        vertex_normal.push(convert_normal(&raw, *inorm));
                        vertex_texture.push(convert_texture(&raw, *itex));
                    });
                }
            }
            _ => return Err(ObjError::NonTriangulatedMesh),
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertex_position);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vertex_normal);
    if !vertex_texture.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vertex_texture);
    }

    mesh.set_indices(Some(Indices::U32(indices.into())));

    Ok(())
}

fn convert_position(raw: &RawObj, index: usize) -> [f32; 3] {
    let position = raw.positions[index];
    [position.0, position.1, position.2]
}

fn convert_normal(raw: &RawObj, index: usize) -> [f32; 3] {
    let normal = raw.normals[index];
    [normal.0, normal.1, normal.2]
}

fn convert_texture(raw: &RawObj, index: usize) -> [f32; 2] {
    let tex_coord = raw.tex_coords[index];
    // Flip UV for correct values
    [tex_coord.0, 1.0 - tex_coord.1]
}

/// Simple and inaccurate normal calculation
fn calculate_normal(raw: &RawObj, polygon: &[usize]) -> [f32; 3] {
    use bevy_math::Vec3;

    // Extract triangle
    let triangle: Vec<Vec3> = polygon
        .iter()
        .map(|index| Vec3::from(convert_position(raw, *index)))
        .collect();

    // Calculate normal
    let v1 = triangle[1] - triangle[0];
    let v2 = triangle[2] - triangle[0];
    let n = v1.cross(v2);

    n.to_array()
}
