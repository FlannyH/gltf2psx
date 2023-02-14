#![allow(clippy::identity_op, clippy::too_many_arguments, dead_code)]

use std::{
    fs::File,
    io::{Read, Seek},
    os::windows::prelude::FileExt,
    path::Path,
};

use exoquant::{convert_to_indexed, ditherer, optimizer};
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

    // Loop over each submesh in the model
    for (texture_id, (material_name, mesh)) in model.meshes.into_iter().enumerate() {
        // Create PSX mesh for this submesh
        {
            println!("Creating mesh for '{material_name}'");
            let mut mesh_psx = MeshPSX { verts: Vec::new() };

            // Convert each vertex to a PSX vertex
            for vertex in mesh.verts {
                mesh_psx
                    .verts
                    .push(VertexPSX::from(&vertex, texture_id as u8));
            }

            // Then add this submesh to the array
            model_psx_out.meshes.push(mesh_psx);
        }

        // Create PSX texture collection for this submesh
        {
            // Retrieve material corresponding to this submesh
            let mat: &Material = &model.materials[&material_name];

            // Create texture cell object
            let mut tex_cell = TextureCellPSX {
                texture_data: Vec::new(),
                palette: Vec::new(),
                texture_width: mat.texture.width as u8,
                texture_height: mat.texture.height as u8,
            };

            // Quantize it to 16 colours
            let mut tex_data_exoquant = Vec::new();
            let tex_data_src = &mat.texture.data;
            for pixel in tex_data_src {
                let pixel8 = pixel.to_be_bytes();
                tex_data_exoquant.push(exoquant::Color::new(
                    pixel8[0], pixel8[1], pixel8[2], pixel8[3],
                ));
            }
            let (palette, indexed_data) = convert_to_indexed(
                &tex_data_exoquant,
                mat.texture.width,
                16,
                &optimizer::KMeans,
                &ditherer::Ordered,
            );
            for color in palette {
                let color: u16 = (color.a as u16).clamp(0, 1) << 15
                    | (color.b as u16).clamp(0, 31) << 10
                    | (color.g as u16).clamp(0, 31) << 5
                    | (color.r as u16).clamp(0, 31) << 0;
                tex_cell.palette.push(color);
            }

            // Convert indices to 4 bit
            println!("{:?}", indexed_data.len());
            for i in (0..(mat.texture.width * mat.texture.height)).step_by(2) {
                if (i + 1) < indexed_data.len() {
                    tex_cell
                        .texture_data
                        .push(indexed_data[i + 0] << 4 | indexed_data[i + 1]);
                } else {
                    tex_cell.texture_data.push(0)
                }
            }

            // Add this cell to the collection
            txc_psx_out.texture_cells.push(tex_cell);
            txc_psx_out.texture_names.push(material_name);
        }
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
    let offset_textures = u32::from_le_bytes(buf32);
    validate(file.read(&mut buf32));
    let offset_palettes = u32::from_le_bytes(buf32);
    validate(file.read(&mut buf32));
    let offset_name_table = u32::from_le_bytes(buf32);

    println!("n_texture_cell: {}", n_texture_cell);
    println!("offset_texture_cell_descs: {}", offset_texture_cell_descs);
    println!("offset_textures: {}", offset_textures);
    println!("offset_palettes: {}", offset_palettes);
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
                binary_offset + offset_texture_cell_descs as u64 + (i as u64 * 4),
            ))
            .unwrap();

        // Get the data
        let mut buf64 = [0, 0, 0, 0, 0, 0, 0, 0];
        validate(file.read(&mut buf64));
        let texture_cell = TextureCellBinary::from_bytes(&buf64);

        // Print the data
        println!(
            "Texture {}: offset: {}\tresolution: {}x{}, \tpalette_index: {}",
            i,
            texture_cell.sector_offset_texture as u32 * 2048,
            texture_cell.texture_width,
            texture_cell.texture_height,
            texture_cell.palette_index
        );
    }

    true
}
