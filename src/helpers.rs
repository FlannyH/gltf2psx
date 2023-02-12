use glam::Vec2;

pub fn index_to_coords(index: usize, width: usize) -> glam::Vec2 {
    glam::vec2((index % width) as f32, (index / width) as f32)
}
pub fn coords_to_index(x: usize, y: usize, width: usize) -> usize {
    x + (y * width)
}
pub fn colour_rgb(red: u8, green: u8, blue: u8) -> u32 {
    ((red as u32) << 16) + ((green as u32) << 8) + (blue as u32)
}
pub fn colour_rgba(alpha: u8, red: u8, green: u8, blue: u8) -> u32 {
    ((alpha as u32) << 24) + ((red as u32) << 16) + ((green as u32) << 8) + (blue as u32)
}

pub fn to_argb8(a: u8, r: u8, g: u8, b: u8) -> u32 {
    (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}

pub fn edge_function(v0: Vec2, v1: Vec2, p: Vec2) -> f32 {
    let v0_p = p - v0;
    let v0_v1 = v1 - v0;
    (v0_p.x * v0_v1.y) - (v0_p.y * v0_v1.x)
}

fn point_inside_triangle(v0: Vec2, v1: Vec2, v2: Vec2, p: Vec2) -> bool {
    (edge_function(v0, v1, p) > 0.0)
        && (edge_function(v1, v2, p) > 0.0)
        && (edge_function(v2, v0, p) > 0.0)
}
