#include "resource_manager.h"
#include <fstream>

#include "logger.h"

void ResourceManager::tick(float dt)
{
	//Garbage collection
	curr_timer += dt;
	if (curr_timer > timer_length)
	{
		curr_timer -= timer_length;

		//Unload resources that are scheduled for unload
		for (const auto& [hash, resource] : resources)
		{
			if (resource->scheduled_for_unload)
			{
				switch(resource->resource_type)
				{
				case ResourceType::texture:
					Logger::logf("Unloading resource:\t%s", ((TextureResource*)resource)->name);
					((TextureResource*)resource)->unload();
					break;
				}
				resources.erase(hash);
				break;
			}
		}
	}
}

std::vector<ResourceDebug> ResourceManager::debug_loaded_resources()
{
	std::vector<ResourceDebug> result;
	for (auto resource : resources)
	{

		ResourceDebug rd;
		std::string types[]
		{
			"invalid",
			"texture",
			"model",
			"shader",
		};

		if (resource.second == nullptr)
			continue;

		if ((unsigned int)resource.second->resource_type > 3)
			continue;

		rd.type = types[(int)resource.second->resource_type];
		rd.name = "???";

		switch (resource.second->resource_type)
		{
		case ResourceType::texture:
			rd.name = ((TextureResource*)resource.second)->name;
		}

		result.push_back(rd);
	}
	return result;
}

//If the file can not be leaded, the size will be zero and the data pointer will be nullptr
void ResourceManager::read_file(const std::string& path, int& size_bytes, char*& data, const bool silent)
{
	//Open file
	std::ifstream file_stream(path, std::ios::binary);

	//Is it actually open?
	if (file_stream.is_open() == false)
	{
		if (!silent)	
			printf("[ERROR] Failed to open file '%s'!\n", path.c_str());
		size_bytes = 0;
		data = nullptr;
		return;
	}

	//See how big the file is so we can allocate the right amount of memory
	const auto begin = file_stream.tellg();
	file_stream.seekg(0, std::ifstream::end);
	const auto end = file_stream.tellg();
	const auto size = end - begin;
	size_bytes = size;

	//Allocate memory
	get_allocator_instance()->curr_memory_chunk_label = "file loading - " + path;
	data = static_cast<char*>(get_allocator_instance()->allocate(static_cast<uint32_t>(size)));
	get_allocator_instance()->curr_memory_chunk_label = "untitled";

	//Load file data into that memory
	file_stream.seekg(0, std::ifstream::beg);
	std::vector<unsigned char> buffer(std::istreambuf_iterator<char>(file_stream), {});

	//Is it actually open?
	if (buffer.empty())
	{
		if (!silent)
			printf("[ERROR] Failed to open file '%s'!\n", path.c_str());
		size_bytes = 0;
		data = nullptr;
		return;
	}
	memcpy(data, &buffer[0], size_bytes);
}

DynamicAllocator* ResourceManager::get_allocator_instance()
{
	if (allocator == nullptr)
	{
		allocator = new DynamicAllocator(512 * 1024 * 1024);
	}
	return allocator;
}

DynamicAllocator* ResourceManager::allocator = nullptr;

uint32_t ResourceManager::xorshift(const uint32_t input)
{
	uint32_t output = input;
	output ^= output << 13;
	output ^= output >> 17;
	output ^= output << 5;
	return output;
}

uint32_t ResourceManager::generate_hash_from_string(const std::string& string)
{
	constexpr std::hash<std::string> hasher;
	return static_cast<uint32_t>(hasher(string));
}
