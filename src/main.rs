#![allow(clippy::identity_op, clippy::too_many_arguments, dead_code)]

use std::{
    fs::File,
    io::{Read, Seek},
    os::windows::prelude::FileExt,
    path::Path, iter::Map, collections::HashMap,
};

use exoquant::{convert_to_indexed, ditherer, optimizer, Color};
use glam::Vec3;
use helpers::validate;
use mesh::Model;
use psx_structs::{MeshPSX, TextureCollectionPSX};

use crate::{
    psx_structs::{MeshDesc, ModelPSX, TextureCellBinary, TextureCellPSX, VertexPSX},
    texture::Material,
};

mod helpers;
mod mesh;
mod psx_structs;
mod structs;
mod texture;
use image::{RgbaImage, DynamicImage, Rgba};
const DEBUG_VIEW: bool = false;

fn main() {
    // Get command line arguments and check if we have one
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return;
    }
    let path_in = args[1].clone();

    // If it's a glTF, load it and export a .msh file
    if path_in.ends_with(".gltf") {
        let path_out = path_in.replace(".gltf", "");
        export_msh(path_in, path_out);
        return;
    }

    // If it's a .msh file, debug it
    if path_in.ends_with(".msh") {
        debug_msh(path_in);
        return;
    }

    // If it's a .msh file, debug it
    if path_in.ends_with(".txc") {
        debug_txc(path_in);
    }
}

fn export_msh(path_in: String, path_out: String) {
    // Load the glTF
    let mut model = Model::new();
    model.create_from_gltf(Path::new(path_in.as_str()));

    // Prepare PSX output model
    let mut model_psx_out = ModelPSX::new();
    let mut txc_psx_out = TextureCollectionPSX::new();

    // Make a map based on a grid
    let grid_size = (1.75, 50000.0, 1.75);
    let mut mesh_grid: HashMap<i128, MeshPSX> = HashMap::new();

    // Loop over each submesh in the model
    for (texture_id, (material_name, mesh)) in model.meshes.into_iter().enumerate() {
        // Create PSX mesh for this submesh
        {
            // Convert each triangle to a PSX triangle
            for triangle in mesh.verts.chunks(3) {
                // Find which gridcell this triangle belongs to
                let average_position = (triangle[0].position + triangle[1].position + triangle[2].position) / 3.0;
                let grid_x = (average_position.x / grid_size.0).round() as i32;
                let grid_y = (average_position.y / grid_size.1).round() as i32;
                let grid_z = (average_position.z / grid_size.2).round() as i32;
                let map_entry = (grid_x as i128) | (grid_y as i128) << 32 | (grid_z as i128) << 64;

                // Create entry in grid map if it didn't exist yet
                if mesh_grid.get(&map_entry).is_none() {
                    mesh_grid.insert(map_entry, MeshPSX::new());
                }

                // Add this triangle to that mesh
                let mesh_psx = mesh_grid.get_mut(&map_entry).unwrap();
                mesh_psx.verts.push(VertexPSX::from(&triangle[0], texture_id as u8));
                mesh_psx.verts.push(VertexPSX::from(&triangle[1], texture_id as u8));
                mesh_psx.verts.push(VertexPSX::from(&triangle[2], texture_id as u8));
            }
        }

        // Create PSX texture collection for this submesh
        {
            // Retrieve material corresponding to this submesh
            let mat: &Material = &model.materials[&material_name];

            // For debug purposes, export the textures
            if DEBUG_VIEW {
                let mut pixels = Vec::new();
                for value in &mat.texture.data {
                    pixels.push(((value >> 0) & 0xFF) as u8);
                    pixels.push(((value >> 8) & 0xFF) as u8);
                    pixels.push(((value >> 16) & 0xFF) as u8);
                    pixels.push(((value >> 24) & 0xFF) as u8);
                }
                let image_data = RgbaImage::from_vec(64, 64, pixels).unwrap();
                let output = DynamicImage::ImageRgba8(image_data);
                output.save(format!("{material_name}.png")).unwrap();
            }

            // Create texture cell object
            let mut tex_cell = TextureCellPSX {
                texture_data: Vec::new(),
                palette: Vec::new(),
                texture_width: mat.texture.width as u8,
                texture_height: mat.texture.height as u8,
                avg_color: 0,
            };

            // Quantize it to 16 colours
            let mut tex_data_exoquant = Vec::new();
            let tex_data_src = &mat.texture.data;
            for pixel in tex_data_src {
                let pixel8 = pixel.to_be_bytes();
                tex_data_exoquant.push(exoquant::Color::new(
                    pixel8[3], pixel8[2], pixel8[1], pixel8[0],
                ));
            }
            let (palette, indexed_data) = convert_to_indexed(
                &tex_data_exoquant,
                mat.texture.width,
                16,
                &optimizer::KMeans,
                &ditherer::Ordered,
            );
            let color_b = Color {
                r: (mat.texture.avg_color & 0x000000FF >> 0) as u8,
                g: (mat.texture.avg_color & 0x0000FF00 >> 8) as u8,
                b: (mat.texture.avg_color & 0x00FF0000 >> 16) as u8,
                a: (mat.texture.avg_color & 0xFF000000 >> 24) as u8,
            };
            for fade_level in 0..16 {
                for color in &palette {
                    let color: u16 = (color.a as u16).clamp(0, 1) << 15
                        | ((((fade_level * color_b.b as u16)
                            + ((15 - fade_level) * color.b as u16))
                            / 15)
                            >> 3)
                            .clamp(0, 31)
                            << 10
                        | ((((fade_level * color_b.g as u16)
                            + ((15 - fade_level) * color.g as u16))
                            / 15)
                            >> 3)
                            .clamp(0, 31)
                            << 5
                        | ((((fade_level * color_b.r as u16)
                            + ((15 - fade_level) * color.r as u16))
                            / 15)
                            >> 3)
                            .clamp(0, 31)
                            << 0;
                    tex_cell.palette.push(color);
                }
            }

            // Convert indices to 4 bit
            println!("{:?}", indexed_data.len());
            for i in (0..(mat.texture.width * mat.texture.height)).step_by(2) {
                if (i + 1) < indexed_data.len() {
                    tex_cell
                        .texture_data
                        .push((indexed_data[i + 0] << 4) | (indexed_data[i + 1]));
                } else {
                    tex_cell.texture_data.push(0);
                    tex_cell.texture_data.push(0);
                    tex_cell.texture_data.push(0);
                    tex_cell.texture_data.push(0);
                }
            }

            // Set average color in cell
            tex_cell.avg_color = mat.texture.avg_color;

            // Add this cell to the collection
            txc_psx_out.texture_cells.push(tex_cell);
            txc_psx_out.texture_names.push(material_name);
        }
    }

    // For every grid cell, put it in the model_psx
    for (_, mesh) in mesh_grid {
        model_psx_out.meshes.push(mesh);
    }

    validate(model_psx_out.save(Path::new(&(path_out.clone() + ".msh"))));
    validate(txc_psx_out.save(Path::new(&(path_out + ".txc"))));
}

fn debug_msh(path_in: String) -> bool {
    let mut file = File::open(Path::new(path_in.as_str())).unwrap();
    println!("FMSH file debug");

    // Verify file magic
    let mut buf32 = [0, 0, 0, 0];

    validate(file.read(&mut buf32));
    let file_magic = u32::from_be_bytes(buf32);
    match file_magic != 0x424D9640 {
        true => println!("File magic ok. (\"FMSH\")"),
        false => {
            println!("File magic not ok. Invalid file.");
            return false;
        }
    }

    // Verify number of submeshes
    validate(file.read(&mut buf32));
    let n_submeshes = u32::from_le_bytes(buf32);
    println!("n_submeshes: {n_submeshes}");

    // Get mesh description offset
    validate(file.read(&mut buf32));
    let offset_mesh_desc = u32::from_le_bytes(buf32);
    println!("offset_mesh_desc: {offset_mesh_desc}");

    // Get vertex data offset
    validate(file.read(&mut buf32));
    let offset_vertex_data = u32::from_le_bytes(buf32);
    println!("offset_vertex_data: {offset_vertex_data}");

    // Get the current position - the binary data starts here
    let binary_offset = 16;

    // Check if the offsets are sane
    let start_binary_section = file
        .seek(std::io::SeekFrom::Start(
            binary_offset + offset_vertex_data as u64,
        ))
        .unwrap();
    let end_binary_section = file.seek(std::io::SeekFrom::End(0)).unwrap();
    let number_of_bytes = end_binary_section.overflowing_sub(start_binary_section).0;
    if offset_mesh_desc as u64 > number_of_bytes || offset_vertex_data as u64 > number_of_bytes {
        println!("Offsets are out of bounds! File is unsafe!");
        return false;
    }

    // Binary section starts after this
    // First read all the mesh descriptions
    let mut buf_mesh_desc = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    let mut lowest_vertex_index = 0u16;
    let mut highest_vertex_index = 0u16;

    for submesh_index in 0..n_submeshes {
        validate(file.seek_read(
            &mut buf_mesh_desc,
            binary_offset + offset_mesh_desc as u64 + (submesh_index * 16) as u64,
        ));
        let mesh_desc = MeshDesc::from_bytes(&buf_mesh_desc);
        println!("mesh_descs[{submesh_index}]:");
        println!("\tvertex_start: {}", mesh_desc.vertex_start);
        println!("\tn_vertices: {}", mesh_desc.n_vertices);
        println!("\tx_min, x_max: {}, {}", mesh_desc.x_min, mesh_desc.x_max);
        println!("\ty_min, y_max: {}, {}", mesh_desc.y_min, mesh_desc.y_max);
        println!("\tz_min, z_max: {}, {}", mesh_desc.z_min, mesh_desc.z_max);
        lowest_vertex_index = lowest_vertex_index.min(mesh_desc.vertex_start);
        highest_vertex_index =
            highest_vertex_index.min(mesh_desc.vertex_start + mesh_desc.n_vertices);
    }

    // Check if vertex indices fit inside the binary section
    if (lowest_vertex_index.max(highest_vertex_index) * 12) as u64 > number_of_bytes {
        println!("Vertex data is out of bounds! File is unsafe!");
        return false;
    }

    println!("File is ok.");

    true
}

fn debug_txc(path_in: String) -> bool {
    let mut file = File::open(Path::new(path_in.as_str())).unwrap();
    println!("FTXC file debug");

    // Verify file magic
    let mut buf32 = [0, 0, 0, 0];

    validate(file.read(&mut buf32));
    let file_magic = u32::from_be_bytes(buf32);
    match file_magic != 0x464D5348 {
        true => println!("File magic ok. (\"FTXC\")"),
        false => {
            println!("File magic not ok. Invalid file.");
            return false;
        }
    }

    // Get n_texture_cell
    validate(file.read(&mut buf32));
    let n_texture_cell = u32::from_le_bytes(buf32);
    validate(file.read(&mut buf32));
    let offset_texture_cell_descs = u32::from_le_bytes(buf32);
    validate(file.read(&mut buf32));
    let offset_palettes = u32::from_le_bytes(buf32);
    validate(file.read(&mut buf32));
    let offset_textures = u32::from_le_bytes(buf32);
    validate(file.read(&mut buf32));
    let offset_name_table = u32::from_le_bytes(buf32);

    println!("n_texture_cell: {}", n_texture_cell);
    println!("offset_texture_cell_descs: {}", offset_texture_cell_descs);
    println!("offset_palettes: {}", offset_palettes);
    println!("offset_textures: {}", offset_textures);
    println!("offset_name_table: {}", offset_name_table);

    // Validate offsets
    let binary_offset = 24;
    let start_binary_section = file.seek(std::io::SeekFrom::Start(binary_offset)).unwrap();
    let end_binary_section = file.seek(std::io::SeekFrom::End(0)).unwrap();
    let number_of_bytes = end_binary_section.overflowing_sub(start_binary_section).0;
    if offset_texture_cell_descs as u64 > number_of_bytes
        || offset_textures as u64 > number_of_bytes
        || offset_palettes as u64 > number_of_bytes
        || ((offset_name_table != 0xFFFFFFFFu32) && (offset_name_table as u64 > number_of_bytes))
    {
        println!("Offsets are out of bounds! File is unsafe!");
        return false;
    }

    // Debug each texture cell
    for i in 0..n_texture_cell {
        // Find texture cell
        let _ = file
            .seek(std::io::SeekFrom::Start(
                binary_offset + offset_texture_cell_descs as u64 + (i as u64 * 8),
            ))
            .unwrap();

        // Get the data
        let mut buf64 = [0u8; 8];
        validate(file.read(&mut buf64));
        let texture_cell = TextureCellBinary::from_bytes(&buf64);

        // Print the data
        println!(
            "Texture {}: offset: {}\tresolution: {}x{}, \tpalette_index: {},\tavg_color: {:08X}",
            i,
            texture_cell.sector_offset_texture as u32 * 2048,
            texture_cell.texture_width,
            texture_cell.texture_height,
            texture_cell.palette_index,
            texture_cell.avg_color
        );

        // Find the texture data
        let _ = file
            .seek(std::io::SeekFrom::Start(
                binary_offset + offset_textures as u64 + (i as u64 * 2048),
            ))
            .unwrap();

        // Read the texture data
        let mut texture_indices_4bit = [0u8; 64*64/2];
        validate(file.read(&mut texture_indices_4bit));

        // Find the palette data
        let _ = file
            .seek(std::io::SeekFrom::Start(
                binary_offset + offset_palettes as u64 + (i as u64 * 32 * 16),
            ))
            .unwrap();

        // Read the palette data
        let mut palette_16bit = [0u8; 32];
        validate(file.read(&mut palette_16bit));

        // Loop over each pixel
        let mut pixels = RgbaImage::new(64, 64);
        for y in 0..64 {
            for x in 0..64 {
                let mut pixel = texture_indices_4bit[(x + (y * 64)) / 2];

                // Extract the 4-bit index from the byte
                if (x % 2) == 0 {
                    pixel = (pixel & 0xF0) >> 4;
                } else {
                    pixel = pixel & 0x0F;
                }

                // Look up the color in the palette
                let color = palette_16bit[pixel as usize * 2 + 0] as u16 + ((palette_16bit[pixel as usize * 2 + 1] as u16) << 8);

                // Convert to 32 bit color
                let r = 8 * ((color >> 0) & 0x1F) as u8;
                let g = 8 * ((color >> 5) & 0x1F) as u8;
                let b = 8 * ((color >> 10) & 0x1F) as u8;
                let a = 255 * ((color >> 15) & 0x01) as u8;

                pixels.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
            }
        }

        // Export the image
        let output = DynamicImage::ImageRgba8(pixels);
        output.save(format!("texture{i}.png")).unwrap();
    }

    true
}
