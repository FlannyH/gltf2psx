use std::{path::Path, sync::Arc};

use mesh::Model;
use psx_structs::MeshPSX;

use crate::{
    mesh::Mesh,
    psx_structs::{ModelPSX, VertexPSX},
};

mod helpers;
mod mesh;
mod psx_structs;
mod structs;
mod texture;

fn main() {
    // Get path
    let path_in = String::from("D:/Library/Documents/GitHub/ShooterPSX/assets/level.gltf");
    let path_out = path_in.replace(".gltf", "") + ".msh";

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

    model_psx_out.save(Path::new(&path_out));
}
