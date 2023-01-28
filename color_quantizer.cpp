#include "color_quantizer.h"

#include <algorithm>

int ordered_dithering_lut[4 * 4] = {
    0, 8, 2, 10,
    12, 4, 14, 6,
    3, 11, 1, 9,
    15, 7, 13, 5
};

int calculate_distance_squared(const Pixel32 a, const Pixel32 b) {
    return
        ((int)a.r - (int)b.r) * ((int)a.r - (int)b.r) +
        ((int)a.g - (int)b.g) * ((int)a.g - (int)b.g) +
        ((int)a.b - (int)b.b) * ((int)a.b - (int)b.b);
}

QuantizationOutput quantize_image(Pixel32* image, int w, int h, int n_colors, bool dither_enable, int n_iterations,
                               int n_restarts) {
    // Initiate sessions
    int lowest_total_error = std::numeric_limits<int>::max();
    int lowest_total_error_index = 0;
    vector<Session> sessions;
    sessions.resize(n_restarts);

    for (int restart = 0; restart < n_restarts; ++restart) {
        auto& session = sessions[restart];

        // Initialize a random color palette
        vector<Cluster> clusters;
        clusters.resize(n_colors);
        for (auto& cluster : clusters) {
            cluster.color.r = rand() & 0xFF;
            cluster.color.g = rand() & 0xFF;
            cluster.color.b = rand() & 0xFF;
        }

        for (int iterations = 0; iterations < n_iterations; ++iterations) {
            // Clear the clusters' pixel entries
            for (auto& cluster : clusters) {
                cluster.entries.clear();
            }

            // Loop over each pixel
            for (int y = 0; y < h; ++y) {
                for (int x = 0; x < w; ++x) {
                    int closest_index = 0;
                    int closest_distance = std::numeric_limits<int>::max();
                    const Pixel32& pixel = image[x + y * w];

                    // Find the closest entry for this pixel
                    for (int i = 0; i < n_colors; i++) {
                        int curr_distance = calculate_distance_squared(pixel, clusters[i].color);
                        if (curr_distance < closest_distance) {
                            closest_distance = curr_distance;
                            closest_index = i;
                        }
                    }

                    // Add this pixel to that cluster
                    clusters[closest_index].entries.push_back(pixel);
                }
            }

            // Adjust palette
            for (auto& cluster : clusters) {
                if (cluster.entries.empty()) {
                    continue;
                }

                // Calculate average
                int x = 0;
                int y = 0;
                int z = 0;
                for (auto& entry : cluster.entries) {
                    x += entry.r;
                    y += entry.g;
                    z += entry.b;
                }
                x /= cluster.entries.size();
                y /= cluster.entries.size();
                z /= cluster.entries.size();
                cluster.color.r = x;
                cluster.color.g = y;
                cluster.color.b = z;
            }
        }

        // Extract final palette
        session.final_palette.resize(n_colors);
        for (int i = 0; i < n_colors; ++i) {
            if (clusters[i].entries.empty())
                session.final_palette[i] = { 255, 255, 255 };
            else
                session.final_palette[i] = clusters[i].color;
        }

        // Sort final palette
        std::ranges::sort(session.final_palette, [](Pixel32 a, Pixel32 b)
        {
            return calculate_distance_squared({ 0, 0, 0 }, a) > calculate_distance_squared({ 0, 0, 0 }, b);
        });

        // Create output buffer
        session.output_texture.resize(w * h);
        session.output_pixel.resize(w * h);

        // Loop over each pixel and find the closest index
        vector<int> distances;
        distances.resize(n_colors);

        int min = std::numeric_limits<int>::max();
        int max = std::numeric_limits<int>::min();

        for (int y = 0; y < h; ++y) {
            for (int x = 0; x < w; ++x) {
                const Pixel32& pixel = image[x + y * w];

                // Find the distances to all the colors in the palette
                for (int i = 0; i < n_colors; i++) {
                    distances[i] = sqrt(calculate_distance_squared(pixel, session.final_palette[i]));
                }

                // Find the lowest distance
                int distance1 = std::numeric_limits<int>::max();
                int distance2 = std::numeric_limits<int>::max();
                int closest_index1 = 0;
                int closest_index2 = 0;
                for (int i = 0; i < n_colors; ++i) {
                    if (distances[i] <= distance1) {
                        distance2 = distance1;
                        distance1 = distances[i];
                        closest_index2 = closest_index1;
                        closest_index1 = i;
                    }
                }

                // Get the 2 color values
                Pixel32 color1 = session.final_palette[closest_index1];
                Pixel32 color2 = session.final_palette[closest_index2];

                float t_x = (color1.r == color2.r) ? 0.5f : (float)(image[x + y * w].r - color1.r) / (float)(color2.r - color1.r);
                float t_y = (color1.g == color2.g) ? 0.5f : (float)(image[x + y * w].g - color1.g) / (float)(color2.g - color1.g);
                float t_z = (color1.b == color2.b) ? 0.5f : (float)(image[x + y * w].b - color1.b) / (float)(color2.b - color1.b);
                float t_avg = (t_x + t_y + t_z) / 3;
                t_avg *= 15;

                // Dithering with percentages
                int dither = ((uint8_t)ordered_dithering_lut[(x % 4) + ((y % 4) * 4)]);
                if (dither >= t_avg && dither_enable) {
                    session.output_texture[x + y * w] = closest_index1;
                }
                else {
                    session.output_texture[x + y * w] = closest_index2;
                }

                // Calculate distance between chosen pixel and actual pixel, and accumulate error
                const auto final_pixel = session.final_palette[session.output_texture[x + y * w]];
                session.total_error += calculate_distance_squared(final_pixel, image[x + y * w]);

                // For the output image
                session.output_pixel[x + y * w] = final_pixel;
            }
        }

        // Save the best one
        if (session.total_error < lowest_total_error) {
            lowest_total_error = session.total_error;
            lowest_total_error_index = restart;
        }
    }

    // Generate output
    QuantizationOutput output = {
        sessions[lowest_total_error_index].output_pixel,
        sessions[lowest_total_error_index].output_texture,
        sessions[lowest_total_error_index].final_palette,
    };
    return output;
}
