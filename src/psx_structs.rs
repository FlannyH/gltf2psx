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

#[derive(Clone, Copy)]
pub struct MeshDesc {
    pub vertex_start: u16,
    pub n_vertices  : u16,
    pub x_min       : i16,
    pub x_max       : i16,
    pub y_min       : i16,
    pub y_max       : i16,
    pub z_min       : i16,
    pub z_max       : i16,
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
        let mut mesh_descs = Vec::<MeshDesc>::new();

        // For each submesh, add the vertices to the array, and store 32-bit offsets to the start of each of them
        for mesh in self.meshes.as_slice() {
            // Find AABB extremes
            let mut x_max= -32768;
            let mut x_min= 32767;
            let mut y_max= -32768;
            let mut y_min= 32767;
            let mut z_max= -32768;
            let mut z_min= 32767;
            for vertex in &mesh.verts {
                x_max = x_max.max(vertex.pos_x);
                x_min = x_min.min(vertex.pos_x);
                y_max = y_max.max(vertex.pos_y);
                y_min = y_min.min(vertex.pos_y);
                z_max = z_max.max(vertex.pos_z);
                z_min = z_min.min(vertex.pos_z);
            }

            mesh_descs.push(MeshDesc{
                vertex_start: raw_vertex_data.len() as u16,
                n_vertices: mesh.verts.len() as u16,
                x_min, x_max,
                y_min, y_max,
                z_min, z_max,
            });
            for vertex in &mesh.verts {
                raw_vertex_data.push(*vertex);
            }
        }

        // Open output file
        let mut file = File::create(path)?;

        // Write file magic
        file.write("FMSH".as_bytes());

        // Write number of submeshes
        file.write(&(self.meshes.len() as u32).to_le_bytes());

        // The offset into the MeshDesc array is zero here
        // Let's just define it to always be this way
        file.write(&(0u32).to_le_bytes());

        // The vertex data is stored right after the MeshDesc array, but it's aligned to 4 bytes so the PS1 doesn't crap all over itself trying to load it
        let vertex_data_offset = (mesh_descs.len() * 16 + 0x03) & !0x03;
        let delta_offset = vertex_data_offset - mesh_descs.len() * 16;

        // Write the offset to the vertex data
        file.write(&(vertex_data_offset as u32).to_le_bytes());

        for value in mesh_descs {
            file.write(&value.vertex_start.to_le_bytes());
            file.write(&value.n_vertices.to_le_bytes());
            file.write(&value.x_min.to_le_bytes());
            file.write(&value.x_max.to_le_bytes());
            file.write(&value.y_min.to_le_bytes());
            file.write(&value.y_max.to_le_bytes());
            file.write(&value.z_min.to_le_bytes());
            file.write(&value.z_max.to_le_bytes());
        }

        for _ in 0..delta_offset {
            file.write(&[0x69]);
        }

        for vertex in raw_vertex_data {
            file.write(&vertex.get_bytes());
        }

        Ok(())
    }
}

impl MeshDesc {
    pub fn from_bytes(buffer: &[u8]) -> Self {
        let mesh_desc = unsafe { &*(buffer.as_ptr() as *const MeshDesc) };
        *mesh_desc
    }
}