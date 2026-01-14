use crate::ObjSettings;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};

pub(crate) struct MeshConverter {
    indicies: wobj::Indicies,
    verticies: wobj::Vertices,
}

impl MeshConverter {
    pub fn convert(&self, settings: &ObjSettings) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        mesh.insert_indices(Indices::U32(self.indices()));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.position());

        if self.has_uv() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uv());
        }

        if self.has_normal() && !settings.force_compute_normals {
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normal());
        } else if settings.prefer_flat_normals {
            mesh.duplicate_vertices();
            mesh.compute_flat_normals();
        } else {
            mesh.compute_normals();
        }

        mesh
    }

    pub fn new(indicies: wobj::Indicies, verticies: wobj::Vertices) -> Self {
        Self { indicies, verticies }
    }

    fn indices(&self) -> Vec<u32> {
        self.indicies.0.iter().map(|i| *i as u32).collect()
    }

    fn position(&self) -> Vec<[f32; 3]> {
        self.verticies.positions.clone()
    }

    fn has_normal(&self) -> bool {
        self.verticies.normals.is_some()
    }

    fn normal(&self) -> Vec<[f32; 3]> {
        if let Some(normals) = &self.verticies.normals {
            normals.clone()
        } else {
            Vec::new()
        }
    }

    fn has_uv(&self) -> bool {
        self.verticies.uvs.is_some()
    }

    fn uv(&self) -> Vec<[f32; 2]> {
        if let Some(uvs) = &self.verticies.uvs {
            uvs.iter().map(|uv| [uv[0], 1.0 - uv[1]]).collect()
        } else {
            Vec::new()
        }
    }
}
