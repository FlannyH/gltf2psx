use std::{fs::File, io::Write, path::Path};

use crate::{helpers::validate, structs::Vertex};

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
    pub n_vertices: u16,
    pub x_min: i16,
    pub x_max: i16,
    pub y_min: i16,
    pub y_max: i16,
    pub z_min: i16,
    pub z_max: i16,
}

pub struct TextureCollectionPSX {
    pub texture_cells: Vec<TextureCellPSX>,
    pub texture_names: Vec<String>,
}

pub struct TextureCellPSX {
    pub texture_data: Vec<u8>,
    pub palette: Vec<u16>,
    pub texture_width: u8,
    pub texture_height: u8,
    pub avg_color: u32,
}

#[derive(Clone, Copy)]
pub struct TextureCellBinary {
    pub sector_offset_texture: u8,
    pub palette_index: u8,
    pub texture_width: u8,
    pub texture_height: u8,
}

impl VertexPSX {
    pub fn from(vertex: &Vertex, texture_id: u8) -> VertexPSX {
        VertexPSX {
            pos_x: (256.0 * vertex.position.x).clamp(-32768.0, 32767.0) as i16,
            pos_y: (-256.0 * vertex.position.y).clamp(-32768.0, 32767.0) as i16,
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

    pub fn save(&self, path: &Path) -> std::io::Result<usize> {
        // Create binary array of data
        let mut raw_vertex_data = Vec::<VertexPSX>::new();
        let mut mesh_descs = Vec::<MeshDesc>::new();

        // For each submesh, add the vertices to the array, and store 32-bit offsets to the start of each of them
        for mesh in self.meshes.as_slice() {
            // Find AABB extremes
            let mut x_max = -32768;
            let mut x_min = 32767;
            let mut y_max = -32768;
            let mut y_min = 32767;
            let mut z_max = -32768;
            let mut z_min = 32767;
            for vertex in &mesh.verts {
                x_max = x_max.max(vertex.pos_x);
                x_min = x_min.min(vertex.pos_x);
                y_max = y_max.max(vertex.pos_y);
                y_min = y_min.min(vertex.pos_y);
                z_max = z_max.max(vertex.pos_z);
                z_min = z_min.min(vertex.pos_z);
            }

            mesh_descs.push(MeshDesc {
                vertex_start: raw_vertex_data.len() as u16,
                n_vertices: mesh.verts.len() as u16,
                x_min,
                x_max,
                y_min,
                y_max,
                z_min,
                z_max,
            });
            for vertex in &mesh.verts {
                raw_vertex_data.push(*vertex);
            }
        }

        // Open output file
        let mut file = File::create(path)?;

        // Write file magic
        validate(file.write("FMSH".as_bytes()));

        // Write number of submeshes
        validate(file.write(&(self.meshes.len() as u32).to_le_bytes()));

        // The offset into the MeshDesc array is zero here
        // Let's just define it to always be this way
        validate(file.write(&(0u32).to_le_bytes()));

        // The vertex data is stored right after the MeshDesc array, but it's aligned to 4 bytes so the PS1 doesn't crap all over itself trying to load it
        let vertex_data_offset = (mesh_descs.len() * 16 + 0x03) & !0x03;
        let delta_offset = vertex_data_offset - mesh_descs.len() * 16;

        // Write the offset to the vertex data
        validate(file.write(&(vertex_data_offset as u32).to_le_bytes()));

        for value in mesh_descs {
            validate(file.write(&value.vertex_start.to_le_bytes()));
            validate(file.write(&value.n_vertices.to_le_bytes()));
            validate(file.write(&value.x_min.to_le_bytes()));
            validate(file.write(&value.x_max.to_le_bytes()));
            validate(file.write(&value.y_min.to_le_bytes()));
            validate(file.write(&value.y_max.to_le_bytes()));
            validate(file.write(&value.z_min.to_le_bytes()));
            validate(file.write(&value.z_max.to_le_bytes()));
        }

        for _ in 0..delta_offset {
            validate(file.write(&[0x69]));
        }

        for vertex in raw_vertex_data {
            validate(file.write(&vertex.get_bytes()));
        }

        Ok(0)
    }
}

impl MeshDesc {
    pub fn from_bytes(buffer: &[u8]) -> Self {
        let a = unsafe { &*(buffer.as_ptr() as *const MeshDesc) };
        *a
    }
}

impl TextureCollectionPSX {
    pub fn new() -> Self {
        TextureCollectionPSX {
            texture_cells: Vec::new(),
            texture_names: Vec::new(),
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<usize> {
        // Open output file
        let mut file = File::create(path)?;

        // Write file magic
        validate(file.write("FTXC".as_bytes()));

        // Write number of texture cells and palettes
        validate(file.write(&(self.texture_cells.len() as u32).to_le_bytes()));

        // Create binary data buffers for each part
        let mut bin_texture_cell_descs: Vec<u8> = Vec::new();
        let mut bin_palettes: Vec<u8> = Vec::new();
        let mut bin_texture_data: Vec<u8> = Vec::new();

        // Populate these buffers
        for i in 0..self.texture_cells.len() {
            let cell = &self.texture_cells[i];
            // Palettes
            {
                let palette = &cell.palette;
                for color in palette {
                    bin_palettes.push(((color >> 0) & 0xFF) as u8);
                    bin_palettes.push(((color >> 8) & 0xFF) as u8);
                }
            }

            // Texture data
            {
                // Add texture to the texture binary
                // It's aligned in such a way that >=64x64 textures are aligned to CD sectors,
                // and anything lower will align to a subdivision of the CD sector
                let grid_align = 2048;

                // Determine the number of bytes to add to align the texture data
                let curr_position = bin_texture_data.len() as u32;
                let n_bytes_to_add =
                    ((curr_position + (grid_align - 1)) & !(grid_align - 1)) - curr_position;

                println!("curr_position: {curr_position}");
                println!("n_bytes_to_add: {n_bytes_to_add}");

                // Pad to this align
                bin_texture_data.resize(bin_texture_data.len() + n_bytes_to_add as usize, 0);

                // Add the texture data to the binary array
                bin_texture_data.extend(&cell.texture_data);

                // Write texture offset
                bin_texture_cell_descs.push(((curr_position + n_bytes_to_add) / 2048) as u8);

                // Write palette index
                bin_texture_cell_descs.extend_from_slice(&(i as u8).to_le_bytes());

                // Write texture dimensions
                bin_texture_cell_descs.push(cell.texture_width);
                bin_texture_cell_descs.push(cell.texture_height);

                // Write texture dimensions
                bin_texture_cell_descs.extend_from_slice(&cell.avg_color.to_le_bytes());
            }
        }

        // I guess we can just write these in any order at this point, since the offsets will be stored in the main header
        let mut cursor: u32 = 0;

        // Write offset to texture cell descs
        validate(file.write(&(cursor).to_le_bytes()));
        cursor += bin_texture_cell_descs.len() as u32;

        // Write offset to palettes
        validate(file.write(&(cursor).to_le_bytes()));
        cursor += bin_palettes.len() as u32;

        // Align the texture data to a CD sector. This allows for some neat optimizations
        let real_cursor = cursor + 24;
        let bytes_to_pad = ((real_cursor + 2047) & !2047) - real_cursor;
        cursor += bytes_to_pad;

        // Write offset to textures
        validate(file.write(&(cursor).to_le_bytes()));
        //cursor += bin_texture_data.len() as u32;

        // todo: name table
        validate(file.write(&(0u32).to_le_bytes()));

        // Write the raw buffers now, in the right order
        validate(file.write(bin_texture_cell_descs.as_slice()));
        validate(file.write(bin_palettes.as_slice()));

        // Pad with zeroes
        for _ in 0..bytes_to_pad {
            validate(file.write(&[0u8]));
        }

        validate(file.write(bin_texture_data.as_slice()));

        Ok(0)
    }
}

impl TextureCellBinary {
    pub fn from_bytes(buffer: &[u8]) -> Self {
        let a = unsafe { &*(buffer.as_ptr() as *const Self) };
        *a
    }
}
