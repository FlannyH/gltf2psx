#pragma once
#include <vector>
#include "resource_handler_structs.h"

using std::vector;

struct Session {
    vector<uint8_t> output_texture;
    vector<Pixel32> output_pixel;
    vector<Pixel32> final_palette;
    int total_error = 0;
};

struct Cluster {
    Pixel32 color;
    vector<Pixel32> entries;
};

struct QuantizationOutput {
    vector<Pixel32> quantized_pixels;
    vector<uint8_t> indices;
    vector<Pixel32> palette;
};

int calculate_distance_squared(const Pixel32 a, const Pixel32 b);

QuantizationOutput quantize_image(Pixel32* image, int w, int h, int n_colors, bool dither_enable = true,
                               int n_iterations = 32, int n_restarts = 8);