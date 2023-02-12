use crate::helpers::*;
use std::path::Path;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    pub data: Vec<u32>,
    pub mipmap_offsets: Vec<usize>,
}

#[derive(PartialEq)]
pub enum FilterMode {
    Point,
    Linear,
}

pub enum WrapMode {
    Repeat,
    Mirror,
    Clamp,
}

pub struct Sampler {
    pub filter_mode_mag: FilterMode,
    pub filter_mode_min: FilterMode,
    pub filter_mode_mipmap: FilterMode,
    pub wrap_mode_s: WrapMode,
    pub wrap_mode_t: WrapMode,
    pub mipmap_enabled: bool,
}

pub struct Material {
    pub texture: Texture,
    pub sampler: Sampler,
}

#[derive(Clone)]
enum PixelComp {
    Skip,
    Red,
    Green,
    Blue,
    Alpha,
}

impl Texture {
    pub fn load(path: &Path) -> Self {
        //Load image
        let loaded_image = stb_image::image::load(path);

        //Map the image data to argb8 format
        if let stb_image::image::LoadResult::ImageU8(image) = loaded_image {
            if image.depth == 4 {
                let data = (0..image.data.len() / 4)
                    .map(|id| {
                        colour_rgba(
                            image.data[id * 4 + 3],
                            image.data[id * 4],
                            image.data[id * 4 + 1],
                            image.data[id * 4 + 2],
                        )
                    })
                    .collect();
                Self {
                    width: image.width,
                    height: image.height,
                    depth: image.depth,
                    data,
                    mipmap_offsets: vec![0; 1],
                }
            } else if image.depth == 3 {
                let data = (0..image.data.len() / 3)
                    .map(|id| {
                        colour_rgba(
                            255,
                            image.data[id * 3],
                            image.data[id * 3 + 1],
                            image.data[id * 3 + 2],
                        )
                    })
                    .collect();
                Self {
                    width: image.width,
                    height: image.height,
                    depth: image.depth,
                    data,
                    mipmap_offsets: vec![0; 1],
                }
            } else {
                panic!("Unsupported texture type");
            }
        } else {
            panic!("Unsupported texture type");
        }
    }

    //Get ARGB value from a UV coordinate
    pub fn argb_at_uv(
        &self,
        u: f32,
        v: f32,
        mip_level: usize,
        is_mag: bool,
        material: &Material,
    ) -> u32 {
        let u = match material.sampler.wrap_mode_s {
            WrapMode::Repeat => u - u.floor(), // Repeat - like a saw wave
            WrapMode::Mirror => 2.0 * (u * 0.5 - (u * 0.5 + 0.5).floor()).abs(), // Mirror - like a triangle wave
            WrapMode::Clamp => u.clamp(0.0, 1.0 - f32::EPSILON),
        };
        let v = match material.sampler.wrap_mode_s {
            WrapMode::Repeat => v - v.floor(), // Repeat - like a saw wave
            WrapMode::Mirror => 2.0 * (v * 0.5 - (v * 0.5 + 0.5).floor()).abs(), // Mirror - like a triangle wave
            WrapMode::Clamp => v.clamp(0.0, 1.0 - f32::EPSILON),
        };

        let (u, v) = (
            u * (self.width >> mip_level) as f32,
            v * (self.height >> mip_level) as f32,
        );

        if (is_mag && material.sampler.filter_mode_mag == FilterMode::Linear)
            || (!is_mag && material.sampler.filter_mode_min == FilterMode::Linear)
        {
            // Find weights
            let u1 = u.floor();
            let u2 = u.ceil();
            let v1 = v.floor();
            let v2 = v.ceil();
            let weight4 = ((u - u1) * (v - v1)) / ((u2 - u1) * (v2 - v1));
            let weight3 = ((u2 - u) * (v - v1)) / ((u2 - u1) * (v2 - v1));
            let weight2 = ((u - u1) * (v2 - v)) / ((u2 - u1) * (v2 - v1));
            let weight1 = ((u2 - u) * (v2 - v)) / ((u2 - u1) * (v2 - v1));

            // Sample texture
            let index1 = coords_to_index(u1 as usize, v1 as usize, self.width >> mip_level);
            let index2 = coords_to_index(
                (u2 as usize) % (self.width >> mip_level),
                v1 as usize,
                self.width >> mip_level,
            );
            let index3 = coords_to_index(
                u1 as usize,
                (v2 as usize) % (self.height >> mip_level),
                self.width >> mip_level,
            );
            let index4 = coords_to_index(
                (u2 as usize) % (self.width >> mip_level),
                (v2 as usize) % (self.height >> mip_level),
                self.width >> mip_level,
            );

            average_four_pixels(
                self.data[self.mipmap_offsets[mip_level] + index1],
                self.data[self.mipmap_offsets[mip_level] + index2],
                self.data[self.mipmap_offsets[mip_level] + index3],
                self.data[self.mipmap_offsets[mip_level] + index4],
                weight1,
                weight2,
                weight3,
                weight4,
            )
        } else {
            let index = coords_to_index(u as usize, v as usize, self.width >> mip_level);
            self.data[self.mipmap_offsets[mip_level] + index]
        }
    }

    pub fn load_texture_from_gltf_image(image: &gltf::image::Data) -> Texture {
        // Get pixel swizzle pattern
        let swizzle_pattern = match image.format {
            gltf::image::Format::R8 => vec![PixelComp::Red],
            gltf::image::Format::R8G8 => vec![PixelComp::Red, PixelComp::Green],
            gltf::image::Format::R8G8B8 => vec![PixelComp::Red, PixelComp::Green, PixelComp::Blue],
            gltf::image::Format::R8G8B8A8 => vec![
                PixelComp::Red,
                PixelComp::Green,
                PixelComp::Blue,
                PixelComp::Alpha,
            ],
            gltf::image::Format::R16 => vec![PixelComp::Skip, PixelComp::Red],
            gltf::image::Format::R16G16 => vec![
                PixelComp::Skip,
                PixelComp::Red,
                PixelComp::Skip,
                PixelComp::Green,
            ],
            gltf::image::Format::R16G16B16 => vec![
                PixelComp::Skip,
                PixelComp::Red,
                PixelComp::Skip,
                PixelComp::Green,
                PixelComp::Skip,
                PixelComp::Blue,
            ],
            gltf::image::Format::R16G16B16A16 => vec![
                PixelComp::Skip,
                PixelComp::Red,
                PixelComp::Skip,
                PixelComp::Green,
                PixelComp::Skip,
                PixelComp::Blue,
                PixelComp::Skip,
                PixelComp::Alpha,
            ],
            gltf::image::Format::R32G32B32FLOAT => todo!(),
            gltf::image::Format::R32G32B32A32FLOAT => todo!(),
        };
        Texture {
            width: image.width as usize,
            height: image.height as usize,
            depth: 4,
            data: {
                let mut data = Vec::<u32>::new();
                for i in (0..image.pixels.len()).step_by(swizzle_pattern.len()) {
                    let mut new_pixel = 0xFFFFFFFFu32;
                    for (comp, entry) in swizzle_pattern.iter().enumerate() {
                        match entry {
                            PixelComp::Skip => {}
                            PixelComp::Red => {
                                new_pixel = new_pixel & 0xFFFFFF00 | image.pixels[i + comp] as u32
                            }
                            PixelComp::Green => {
                                new_pixel =
                                    new_pixel & 0xFFFF00FF | (image.pixels[i + comp] as u32) << 8
                            }
                            PixelComp::Blue => {
                                new_pixel =
                                    new_pixel & 0xFF00FFFF | (image.pixels[i + comp] as u32) << 16
                            }
                            PixelComp::Alpha => {
                                new_pixel =
                                    new_pixel & 0x00FFFFFF | (image.pixels[i + comp] as u32) << 24
                            }
                        }
                    }
                    data.push(new_pixel);
                }
                data
            },
            mipmap_offsets: vec![0; 1],
        }
    }

    pub fn generate_mipmaps(&mut self) {
        // Set up first target
        let mut src_offset = 0;

        // Iterate until dst_width is 0
        let mut i = 0;
        loop {
            // Calculate resolutions
            let src_width = self.width >> i;
            let src_height = self.height >> i;
            let dst_width = self.width >> (i + 1);
            let dst_height = self.height >> (i + 1);

            if src_width == 1 || src_height == 1 {
                break;
            }

            // Create new texture vector for simplicity sake
            let mut new_mipmap = Vec::<u32>::new();
            for y in 0..dst_height {
                for x in 0..dst_width {
                    // Sample 4 pixels from the source and combine them into one
                    let pixel_sample1 =
                        self.data[src_offset + ((x * 2) + 0) + ((y * 2) + 0) * src_width];
                    let pixel_sample2 =
                        self.data[src_offset + ((x * 2) + 1) + ((y * 2) + 0) * src_width];
                    let pixel_sample3 =
                        self.data[src_offset + ((x * 2) + 0) + ((y * 2) + 1) * src_width];
                    let pixel_sample4 =
                        self.data[src_offset + ((x * 2) + 1) + ((y * 2) + 1) * src_width];
                    let pixel_output = average_four_pixels(
                        pixel_sample1,
                        pixel_sample2,
                        pixel_sample3,
                        pixel_sample4,
                        0.25,
                        0.25,
                        0.25,
                        0.25,
                    );

                    // Write it to the output buffer
                    new_mipmap.push(pixel_output);
                }
            }

            // Store the mipmap
            let new_mipmap_offset = self.data.len();
            self.mipmap_offsets.push(new_mipmap_offset);
            self.data.append(&mut new_mipmap);

            // Move to the mipmap we just created
            src_offset = new_mipmap_offset;
            i += 1;
        }
    }
}

fn average_four_pixels(
    pixel_sample1: u32,
    pixel_sample2: u32,
    pixel_sample3: u32,
    pixel_sample4: u32,
    weight1: f32,
    weight2: f32,
    weight3: f32,
    weight4: f32,
) -> u32 {
    let r = (((pixel_sample1 >> 0) & 0xFF) as f32) * weight1
        + (((pixel_sample2 >> 0) & 0xFF) as f32) * weight2
        + (((pixel_sample3 >> 0) & 0xFF) as f32) * weight3
        + (((pixel_sample4 >> 0) & 0xFF) as f32) * weight4;
    let g = (((pixel_sample1 >> 8) & 0xFF) as f32) * weight1
        + (((pixel_sample2 >> 8) & 0xFF) as f32) * weight2
        + (((pixel_sample3 >> 8) & 0xFF) as f32) * weight3
        + (((pixel_sample4 >> 8) & 0xFF) as f32) * weight4;
    let b = (((pixel_sample1 >> 16) & 0xFF) as f32) * weight1
        + (((pixel_sample2 >> 16) & 0xFF) as f32) * weight2
        + (((pixel_sample3 >> 16) & 0xFF) as f32) * weight3
        + (((pixel_sample4 >> 16) & 0xFF) as f32) * weight4;
    let a = (((pixel_sample1 >> 24) & 0xFF) as f32) * weight1
        + (((pixel_sample2 >> 24) & 0xFF) as f32) * weight2
        + (((pixel_sample3 >> 24) & 0xFF) as f32) * weight3
        + (((pixel_sample4 >> 24) & 0xFF) as f32) * weight4;
    ((r.clamp(0.0, 255.0) as u32) << 0)
        | ((g.clamp(0.0, 255.0) as u32) << 8)
        | ((b.clamp(0.0, 255.0) as u32) << 16)
        | ((a.clamp(0.0, 255.0) as u32) << 24)
}
