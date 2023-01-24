#include <cstdio>
#include <fstream>

#include "resource_manager.h"

struct VertexPS1 {
    int16_t x, y, z;
    uint8_t r, g, b;
    uint8_t u, v, pad;
};

int main(int argc, char** argv) {
    if (argc < 2)
        exit(1);

    std::string file_to_load = argv[1];

    // Get resource
    ResourceManager resource_manager;
    auto handle = resource_manager.load_resource_from_disk<ModelResource>(file_to_load);
    auto resource = resource_manager.get_resource<ModelResource>(handle);

    // Create vertex array
    bool reverse_winding_order = false;
    std::vector<VertexPS1> verts;
    verts.resize(resource->meshes->n_verts);
    for (size_t i = 0; i < resource->meshes->n_verts; ++i) {
        int real_i = i;
        if (reverse_winding_order) {
            constexpr int mapping[] = { 0, +1, -1 };
            real_i += mapping[real_i % 3];
        }
        verts[real_i] = {
            static_cast<int16_t>(256 * resource->meshes->verts[i].position.x),
            static_cast<int16_t>(-256 * resource->meshes->verts[i].position.y),
            static_cast<int16_t>(256 * resource->meshes->verts[i].position.z),
            static_cast<uint8_t>(resource->meshes->verts[i].colour.r * 255),
            static_cast<uint8_t>(resource->meshes->verts[i].colour.g * 255),
            static_cast<uint8_t>(resource->meshes->verts[i].colour.b * 255),
            static_cast<uint8_t>(255 * resource->meshes->verts[i].texcoord.s),
            static_cast<uint8_t>(255 * resource->meshes->verts[i].texcoord.t),
            0
        };
    }

    // Generate PS1 mesh name
    std::string output_file_path = file_to_load.substr(0, file_to_load.find_last_of('.')) + ".msh";

    // Export to PS1 format
    std::ofstream out_file((output_file_path).c_str(), std::ios::out | std::ios::binary);
    if (!out_file.is_open()) {
        exit(-1);
    }

    // Write number of vertices
    uint16_t n_verts = static_cast<uint16_t>(resource->meshes->n_verts);
    out_file.write(reinterpret_cast<char*>(&n_verts), sizeof(n_verts));

    // Write vertex array
    out_file.write(reinterpret_cast<char*>(verts.data()), sizeof(VertexPS1) * verts.size());

    // Close file
    out_file.close();

    printf("test\n");
}
