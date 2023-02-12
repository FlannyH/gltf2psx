use std::{sync::Arc, path::Path};

use mesh::Model;

mod mesh;
mod structs;
mod texture;
mod helpers;

fn main() {
    let path = String::from("D:/Library/Documents/GitHub/ShooterPSX/assets/level.gltf");
    let mut model = Model::new();
    model.create_from_gltf(Path::new(path.as_str()));
    println!("{}", model.meshes.len());
}
