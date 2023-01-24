#pragma once
#include <cstdint>
#include <string>
#include <unordered_map>

#include "resource_handler_structs.h"

//#define NORMAL_ALLOC
#define DEBUG

class DynamicAllocator
{
public:
	DynamicAllocator(const uint32_t size) { init(size); }
	void init(uint32_t size);
	void* allocate(uint32_t size, uint32_t align = 8);
	void release(void* pointer);
	void* reallocate(void* pointer, uint32_t size, uint32_t align = 8);
	void debug_memory();
	std::vector<MemoryChunk> get_memory_chunk_list();

	std::string curr_memory_chunk_label = "unknown";
	std::unordered_map<void*, std::string> memory_labels;

private:
	void* block_start = nullptr;
	uint32_t block_size = 0;
	std::unordered_map<void*, std::string> chunk_names;
};

