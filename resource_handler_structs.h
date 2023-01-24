#pragma once
#include <cstdint>
#include <string>

enum class ResourceType;

struct ResourceHandle
{
	uint32_t hash;
	ResourceType type;
	bool operator ==(const ResourceHandle& rhs) const { return hash == rhs.hash && type == rhs.type; }
	bool operator !=(const ResourceHandle& rhs) const { return hash != rhs.hash && type != rhs.type; }
};

struct Pixel32
{
	uint8_t r = 255;
	uint8_t g = 255;
	uint8_t b = 255;
	uint8_t a = 255;
};

struct MemoryChunk
{
	std::string name;
	void* pointer;
	uint32_t size;
	bool is_free;
};