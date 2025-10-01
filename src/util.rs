use crate::ObjSettings;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};

pub(crate) struct MeshConverter {
    meshes: Vec<tobj::Mesh>,
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

    fn new(meshes: Vec<tobj::Mesh>) -> Self {
        Self { meshes }
    }

    fn indices(&self) -> Vec<u32> {
        let count = self.meshes.iter().map(|m| m.indices.len()).sum();
        let mut data = Vec::with_capacity(count);
        let mut offset = 0;

        for mesh in &self.meshes {
            data.extend(mesh.indices.iter().map(|i| i + offset));
            offset += (mesh.positions.len() / 3) as u32;
        }

        data
    }

    fn position(&self) -> Vec<[f32; 3]> {
        let count = self.meshes.iter().map(|m| m.positions.len() / 3).sum();
        let mut data = Vec::with_capacity(count);

        for mesh in &self.meshes {
            data.append(&mut convert_vec3(&mesh.positions));
        }

        data
    }

    fn has_normal(&self) -> bool {
        !self.meshes.iter().any(|m| m.normals.is_empty())
    }

    fn normal(&self) -> Vec<[f32; 3]> {
        let count = self.meshes.iter().map(|m| m.normals.len() / 3).sum();
        let mut data = Vec::with_capacity(count);

        for mesh in &self.meshes {
            data.append(&mut convert_vec3(&mesh.normals));
        }

        data
    }

    fn has_uv(&self) -> bool {
        !self.meshes.iter().any(|m| m.texcoords.is_empty())
    }

    fn uv(&self) -> Vec<[f32; 2]> {
        let count = self.meshes.iter().map(|m| m.texcoords.len() / 2).sum();
        let mut data = Vec::with_capacity(count);

        for mesh in &self.meshes {
            data.append(&mut convert_uv(&mesh.texcoords));
        }

        data
    }
}

impl From<tobj::Model> for MeshConverter {
    fn from(value: tobj::Model) -> Self {
        Self::new(vec![value.mesh])
    }
}

impl From<Vec<tobj::Model>> for MeshConverter {
    fn from(value: Vec<tobj::Model>) -> Self {
        Self::new(value.into_iter().map(|v| v.mesh).collect())
    }
}

fn convert_vec3(vec: &[f32]) -> Vec<[f32; 3]> {
    vec.chunks_exact(3).map(|v| [v[0], v[1], v[2]]).collect()
}

fn convert_uv(uv: &[f32]) -> Vec<[f32; 2]> {
    uv.chunks_exact(2).map(|t| [t[0], 1.0 - t[1]]).collect()
}
