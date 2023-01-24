#pragma once
#include <cstdint>
#include <string>
#include <unordered_map>

#include "common_defines.h"
#include "dynamic_allocator.h"
#include "resource_handler_structs.h"
#include "resources.h"
#include "logger.h"

struct RawResource;
struct ResourceDebug
{
	std::string type;
	std::string name;
};

class ResourceManager
{
public:
	ResourceManager() {}
	template <class T>
	ResourceHandle load_resource_from_disk(std::string path);
	template <class T>
	ResourceHandle load_resource_from_buffer(std::string name, T* buffer_data);
	void tick(float dt);
	static void read_file(const std::string& path, int& size_bytes, char*& data, bool silent = false);
	template <class T>
	T* get_resource(ResourceHandle handle);
	static DynamicAllocator* get_allocator_instance();
	static DynamicAllocator* allocator;
	static uint32_t generate_hash_from_string(const std::string& string);
	std::vector<ResourceDebug> debug_loaded_resources();

private:
	int curr_resource_index = 0;
	float curr_timer = -10.0f;
	const float timer_length = 0.05f;
	static uint32_t xorshift(uint32_t input);
	std::unordered_map<uint32_t, RawResource*> resources;
};

template <class T>
ResourceHandle ResourceManager::load_resource_from_disk(std::string path)
{
	//Load resource
	get_allocator_instance()->curr_memory_chunk_label = T::name_string() + " - " + path;
	T* resource = static_cast<T*>(dynamic_allocate(sizeof(T), alignof(T)));
	get_allocator_instance()->curr_memory_chunk_label = "unknown";
	bool success = resource->load(path, this);
	if (!success)
	{
		Logger::logf("error loading %s", path.c_str());
		dynamic_free(resource);
		return { 0, ResourceType::invalid };
	}

	//Generate, set, and return handle
	ResourceHandle handle;
	handle.hash = generate_hash_from_string(path);
	handle.type = resource->resource_type;
	if (resources.find(handle.hash) != resources.end())
	{
		((T*)resources[handle.hash])->unload();
	}
	resources[handle.hash] = (RawResource*)resource;
	return handle;
}

template <class T>
ResourceHandle ResourceManager::load_resource_from_buffer(std::string name, T* buffer_data)
{
	//Generate, set, and return handle
	ResourceHandle handle;
	handle.hash = generate_hash_from_string(name);
	handle.type = buffer_data->resource_type;
	resources[handle.hash] = (RawResource*)buffer_data;
	return handle;
}

template <class T>
T* ResourceManager::get_resource(const ResourceHandle handle)
{
	if (handle.hash == 0 || handle.type == ResourceType::invalid)
		return nullptr;
	return (T*)(resources[handle.hash]);
}
