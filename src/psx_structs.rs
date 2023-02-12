use std::{fs::File, io::Write, path::Path};

use crate::structs::Vertex;

#[derive(Clone, Copy)]
pub struct VertexPSX {
    pub pos_x: i16,
    pub pos_y: i16,
    pub pos_z: i16,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub tex_u: u8,
    pub tex_v: u8,
    pub texture_id: u8,
}

pub struct MeshPSX {
    pub verts: Vec<VertexPSX>,
}

pub struct ModelPSX {
    pub meshes: Vec<MeshPSX>,
}

impl VertexPSX {
    pub fn from(vertex: &Vertex, texture_id: u8) -> VertexPSX {
        VertexPSX {
            pos_x: (256.0 * vertex.position.x).clamp(-32768.0, 32767.0) as i16,
            pos_y: (256.0 * vertex.position.y).clamp(-32768.0, 32767.0) as i16,
            pos_z: (256.0 * vertex.position.z).clamp(-32768.0, 32767.0) as i16,
            color_r: (255.0 * vertex.colour.x).clamp(0.0, 255.0) as u8,
            color_g: (255.0 * vertex.colour.y).clamp(0.0, 255.0) as u8,
            color_b: (255.0 * vertex.colour.z).clamp(0.0, 255.0) as u8,
            tex_u: (255.0 * vertex.uv.x) as u8,
            tex_v: (255.0 * vertex.uv.y) as u8,
            texture_id,
        }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.pos_x.to_le_bytes());
        bytes.extend(self.pos_y.to_le_bytes());
        bytes.extend(self.pos_z.to_le_bytes());
        bytes.extend(self.color_r.to_le_bytes());
        bytes.extend(self.color_g.to_le_bytes());
        bytes.extend(self.color_b.to_le_bytes());
        bytes.extend(self.tex_u.to_le_bytes());
        bytes.extend(self.tex_v.to_le_bytes());
        bytes.extend(self.texture_id.to_le_bytes());
        bytes
    }
}

impl MeshPSX {
    pub fn new() -> MeshPSX {
        MeshPSX { verts: Vec::new() }
    }
}

impl ModelPSX {
    pub fn new() -> ModelPSX {
        ModelPSX { meshes: Vec::new() }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        // Create binary array of data
        let mut raw_vertex_data = Vec::<VertexPSX>::new();
        let mut raw_vertex_data_offsets = Vec::<u32>::new();

        // For each submesh, add the vertices to the array, and store 32-bit offsets to the start of each of them
        for mesh in self.meshes.as_slice() {
            raw_vertex_data_offsets.push(raw_vertex_data.len() as u32);
            for vertex in &mesh.verts {
                raw_vertex_data.push(*vertex);
            }
        }

        // Open output file
        let mut file = File::create(path)?;

        // Write, in order:
        // - number of meshes (u32)
        // - offsets in mesh data (u32 * number of meshes)
        // - raw vertex data (VertexPSX * total number of verts)
        file.write(&(self.meshes.len() as u32).to_le_bytes());
        for value in raw_vertex_data_offsets {
            file.write(&value.to_le_bytes());
        }
        for vertex in raw_vertex_data {
            file.write(&vertex.get_bytes());
        }

        Ok(())
    }
}
