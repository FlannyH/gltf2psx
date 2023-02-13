#![allow(clippy::identity_op, clippy::too_many_arguments, dead_code)]

use std::{
    fs::File,
    io::{Read, Seek},
    os::windows::prelude::FileExt,
    path::Path,
};

use helpers::validate;
use mesh::Model;
use psx_structs::MeshPSX;

use crate::psx_structs::{MeshDesc, ModelPSX, VertexPSX};

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
        let path_out = path_in.replace(".gltf", "") + ".msh";
        export_msh(path_in, path_out);
        return;
    }

    // If it's a .msh file, debug it
    if path_in.ends_with(".msh") {
        debug_msh(path_in);
    }
}

fn export_msh(path_in: String, path_out: String) {
    // Load the glTF
    let mut model = Model::new();
    model.create_from_gltf(Path::new(path_in.as_str()));

    // Prepare PSX output model
    let mut model_psx_out = ModelPSX::new();

    // Loop over each submesh in the model
    for (texture_id, (material_name, mesh)) in model.meshes.into_iter().enumerate() {
        // Create PSX mesh for this submesh
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
    validate(model_psx_out.save(Path::new(&path_out)));
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
    let start_binary_section = file
        .seek(std::io::SeekFrom::Start(
            binary_offset + offset_vertex_data as u64,
        ))
        .unwrap();
    let end_binary_section = file.seek(std::io::SeekFrom::End(0)).unwrap();
    let number_of_bytes = end_binary_section - start_binary_section;
    if (lowest_vertex_index.max(highest_vertex_index) * 12) as u64 > number_of_bytes {
        println!("Vertex data is out of bounds! File is unsafe!")
    }

    println!("File seems ok.");

    true
}
