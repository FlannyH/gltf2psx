#pragma once
#include <cstdint>
#include <string>

enum class ResourceType;

struct ResourceHandle {
	uint32_t hash;
	ResourceType type;
	bool operator ==(const ResourceHandle& rhs) const { return hash == rhs.hash && type == rhs.type; }
	bool operator !=(const ResourceHandle& rhs) const { return hash != rhs.hash && type != rhs.type; }
};

struct Pixel32 {
	uint8_t r = 255;
	uint8_t g = 255;
	uint8_t b = 255;
	uint8_t a = 255;
};

struct MemoryChunk {
	std::string name;
	void* pointer;
	uint32_t size;
	bool is_free;
};


struct TextureAtlasHeader {
    uint16_t atlas_width;
    uint16_t atlas_height;
    uint8_t tile_width;
    uint8_t tile_height;
    uint8_t bits_per_pixel;
    uint8_t n_textures;
};