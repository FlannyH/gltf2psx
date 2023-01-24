#pragma once
#include <cstdint>
#include <string>
#include <vector>
#include <tinygltf/tiny_gltf.h>

#include "common_defines.h"
#include "renderer_structs.h"

class ResourceManager;
struct Vertex;
struct Pixel32;

enum class ResourceType
{
	invalid,
	texture,
	model,
	shader,
};

struct alignas(16) RawResource 
{
	ResourceType resource_type;
	bool scheduled_for_unload = false;
};

struct TextureResource
{
	static std::string name_string() { return "TextureResource"; }
	ResourceType resource_type = ResourceType::texture;
	bool scheduled_for_unload = false;
	int width = 0;
	int height = 0;
	Pixel32* data = nullptr;
	char* name = nullptr;
	bool load(std::string path, ResourceManager const* resource_manager, bool silent = false);
	bool load(tinygltf::Image image, ResourceManager const* resource_manager);
	void unload();
	TextureResource(int width_, int height_, Pixel32* data_, char* name_)
	{
		scheduled_for_unload = false;
		resource_type = ResourceType::texture;
		width = width_;
		height = height_;
		data = data_;
		name = name_;
	};
	void schedule_unload()
	{
		scheduled_for_unload = true;
	}
};

struct MaterialResource
{
	static std::string name_string() { return "MaterialResource"; }
	ResourceHandle tex_col{0, ResourceType::invalid};
	ResourceHandle tex_nrm{0, ResourceType::invalid};
	ResourceHandle tex_rgh{0, ResourceType::invalid};
	ResourceHandle tex_mtl{0, ResourceType::invalid};
	ResourceHandle tex_emm{0, ResourceType::invalid};
	glm::vec4 mul_col{1.0f, 1.0f, 1.0f, 1.0f};
	glm::vec3 mul_emm{1.0f, 1.0f, 1.0f};
	glm::vec2 mul_tex{1.0f, 1.0f};
	float mul_nrm = 1.0f;
	float mul_rgh = 1.0f;
	float mul_mtl = 1.0f;
};

struct ModelResource
{
	static std::string name_string() { return "ModelResource"; }
	ResourceType resource_type;
	bool scheduled_for_unload;
	MeshBufferData* meshes;
	MaterialResource* materials;
	int n_meshes;
	int n_materials;
	bool load(std::string path, ResourceManager* resource_manager);
	void unload();
	void traverse_nodes(std::vector<int>& node_indices, tinygltf::Model& model, glm::mat4 local_transform, std::unordered_map<int, MeshBufferData>& primitives_processed);
	void create_vertex_array(MeshBufferData& mesh_out, tinygltf::Primitive primitive_in, tinygltf::Model model, glm::mat4 trans_mat);
};
