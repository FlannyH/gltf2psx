#include "resources.h"
#include "resource_manager.h"

#define STBI_MALLOC(size)				ResourceManager::get_allocator_instance()->allocate(size, 16)
#define STBI_REALLOC(pointer, size)		ResourceManager::get_allocator_instance()->reallocate(pointer, size, 16)
#define STBI_FREE(pointer)				ResourceManager::get_allocator_instance()->release(pointer)
#define TINYGLTF_IMPLEMENTATION
#define STB_IMAGE_IMPLEMENTATION
#define STB_IMAGE_WRITE_IMPLEMENTATION
#define STBI_MSC_SECURE_CRT
#define TINYGLTF_NOEXCEPTION
#define JSON_NOEXCEPTION

#include <glm/vec2.hpp>
#include <glm/vec3.hpp>

#include "color_quantizer.h"
#include "common_defines.h"
#include "dynamic_allocator.h"
#include "logger.h"
#include "renderer_structs.h"
#include "resource_handler_structs.h"
#include "tinygltf/tiny_gltf.h"

bool TextureResource::load(const std::string path, ResourceManager const* resource_manager, bool silent)
{
	//Get name
	ResourceManager::get_allocator_instance()->curr_memory_chunk_label = "TexRes - name - " + path;

	//Load image file
	int channels;
	ResourceManager::get_allocator_instance()->curr_memory_chunk_label = "TexRes - data - " + path;
	uint8_t* u8_data = stbi_load(path.c_str(), &width, &height, &channels, 4);
	ResourceManager::get_allocator_instance()->curr_memory_chunk_label = "unknown";
		
	//Error checking
	if (u8_data == nullptr)
	{
		if (!silent)
			Logger::logf("[ERROR] Image '%s' could not be loaded from disk!\n", path.c_str());
		resource_type = ResourceType::invalid;
		return false;
	}

	//Pad if no alpha channel
	if (channels != 4)
	{
		if (!silent)
			Logger::logf("[ERROR] Image '%s' is not RGBA 32-bit!\n", path.c_str());
	}

	//Set name
	name = static_cast<char*>(dynamic_allocate(path.size() + 1));
	strcpy_s(name, path.size() + 1, path.c_str());

	//Set data
	data = reinterpret_cast<Pixel32*>(u8_data);

	//Return
	resource_type = ResourceType::texture;
	scheduled_for_unload = false;
	return true;
}

bool TextureResource::load(tinygltf::Image image, ResourceManager const* resource_manager)
{
	bool result = true;
	if (image.component != 4)
	{
		Logger::logf("[ERROR] Texture '%s' has %i components, 4 expected!\n", image.name.c_str(), image.component);
		result = false;
	}
	if (image.bits != 8)
	{
		Logger::logf("[ERROR] Texture '%s' has %i bit image data, 8 bit expected!\n", image.name.c_str(), image.bits);
		result = false;
	}
	ResourceManager::get_allocator_instance()->curr_memory_chunk_label = "TexRes - data - " + image.name;
	data = (Pixel32*)dynamic_allocate(image.image.size());
	memcpy(data, image.image.data(), image.image.size());
	height = image.height;
	width = image.width;
	ResourceManager::get_allocator_instance()->curr_memory_chunk_label = "TexRes - name - " + image.name;
	name = (char*)dynamic_allocate(image.uri.size() + 1);
	ResourceManager::get_allocator_instance()->curr_memory_chunk_label = "unknown";
	resource_type = ResourceType::texture;
	scheduled_for_unload = true;
	strcpy_s(name, image.uri.size() + 1, image.uri.c_str());
	return result;
}

void TextureResource::unload()
{
	dynamic_free(data);
	dynamic_free(name);
	dynamic_free(this);
}

bool ModelResource::load(std::string path, ResourceManager* resource_manager)
{
	//Load GLTF file
	tinygltf::TinyGLTF loader;
	tinygltf::Model model;
	std::string error;
	std::string warning;

	loader.LoadASCIIFromFile(&model, &error, &warning, path);

	std::string path_to_model_folder = path.substr(0, path.find_last_of('/')) + "/";

    Pixel32 texture_atlas[512 * 256];
    memset(texture_atlas, 0, 512 * 256 * sizeof(Pixel32));
	std::vector<MaterialResource> materials_vector;
	{
		for (auto& model_material : model.materials)
		{
			//Create material
			MaterialResource pbr_material;

			//Set PBR multipliers
			pbr_material.mul_col = glm::vec4(
				model_material.pbrMetallicRoughness.baseColorFactor[0],
				model_material.pbrMetallicRoughness.baseColorFactor[1],
				model_material.pbrMetallicRoughness.baseColorFactor[2],
				model_material.pbrMetallicRoughness.baseColorFactor[3]
				);

			//Find base colour texture
			int index_texture_colour = model_material.pbrMetallicRoughness.baseColorTexture.index;
			if (index_texture_colour != -1)
			{
				//Find image
				int index_image_colour = model.textures[index_texture_colour].source;
				auto image = model.images[index_image_colour];

                // Copy to texture atlas based on texture index
                // todo: implement color quantizer and write to custom format
                size_t offset_x = (index_image_colour * 64) % 512;
                size_t offset_y = (index_image_colour / 512);
                printf("rect %i: [%i, %i]\n", index_image_colour, offset_x, offset_y);
                Pixel32* pixels = (Pixel32*)image.image.data();
                auto quantized_image = quantize_image(pixels, image.width, image.height, 16, true, 64, 16);
                stbi_write_png((path + "_" + image.name + "_quantized.png").c_str(), image.width, image.height, 4, quantized_image.quantized_pixels.data(), 0);
                stbi_write_png((path + "_" + image.name + "_original.png").c_str(), image.width, image.height, 4, image.image.data(), 0);
			}
			materials_vector.push_back(pbr_material);
		}
	}


	//Go through each node and add it to the primitive vector
	std::unordered_map<int, MeshBufferData> primitives;
	{
		//Get nodes
		auto& scene = model.scenes[model.defaultScene];
		traverse_nodes(scene.nodes, model, glm::mat4(1.0f), primitives);
	}

	//Populate resource
	{
		ResourceManager::get_allocator_instance()->curr_memory_chunk_label = "MdlRes - Mesh - " + path;
		meshes = (MeshBufferData*)dynamic_allocate(sizeof(MeshBufferData) * primitives.size());
		ResourceManager::get_allocator_instance()->curr_memory_chunk_label = "MdlRes - Material - " + path;
		materials = (MaterialResource*)dynamic_allocate(sizeof(MaterialResource) * primitives.size());
		n_meshes = 0;
		n_materials = 0;

		int i = 0;
		for (auto& [material_id, mesh] : primitives)
		{
			meshes[n_meshes] = mesh;
            if (material_id >= 0) materials[n_materials] = materials_vector[material_id];
			n_meshes++;
			n_materials++;
		}
	}

	resource_type = ResourceType::model;
	scheduled_for_unload = false;
	return true;
}

void ModelResource::traverse_nodes(std::vector<int>& node_indices, tinygltf::Model& model, glm::mat4 local_transform, std::unordered_map<int, MeshBufferData>& primitives_processed)
{
	//Loop over all nodes
	for (auto& node_index : node_indices)
	{
		//Get node
		auto& node = model.nodes[node_index];

		//Convert matrix in gltf model to glm::mat4. If the matrix doesn't exist, just set it to identity matrix
		glm::mat4 local_matrix(1.0f);
		int i = 0;
		for (const auto& value : node.matrix) { local_matrix[i / 4][i % 4] = static_cast<float>(value); i++; }
		local_matrix = local_transform * local_matrix;

		//If it has a mesh, process it
		if (node.mesh != -1)
		{
			//Get mesh
			auto& mesh = model.meshes[node.mesh];
			auto& primitives = mesh.primitives;
			for (auto& primitive : primitives)
			{
				printf("Creating vertex array for mesh '%s'\n", node.name.c_str());
				//primitive.material
				MeshBufferData mesh_buffer_data{};
				create_vertex_array(mesh_buffer_data, primitive, model, local_matrix);
				primitives_processed[primitive.material] = mesh_buffer_data;
			}
		}

		//If it has children, process those
		if (!node.children.empty())
		{
			traverse_nodes(node.children, model, local_matrix, primitives_processed);
		}
	}
}

void ModelResource::unload()
{
	//dynamic_free(meshes);
	//dynamic_free(materials);
	//dynamic_free(this);
}

template <typename src_type, typename dst_type>
std::vector<dst_type> pad_components_to_type(src_type* source, size_t n_comp_src, size_t n_comp_dst, size_t n_items, bool normalized) {
    std::vector<dst_type> out_vector;

    // For each item
    for (size_t i = 0; i < n_items; ++i) {
        // For each component
        dst_type to_add;
        for (size_t comp_i = 0; comp_i < n_comp_dst; ++comp_i) {
            // Add first elements
            if (n_comp_src >= comp_i) {
                to_add[comp_i] = static_cast<dst_type::value_type>(source[comp_i + n_comp_src * i]);
                if (normalized) to_add[comp_i] /= std::numeric_limits<src_type>::max();
            }
            else {
                to_add[comp_i] = 0;
            }
        }
        out_vector.push_back(to_add);
        printf("out_vector[%i][0] = %f\n", i, out_vector[i][0]);
    }

    return out_vector;
}

template <typename glm_type>
std::vector<glm_type> gltf_to_glm(void* pointer, tinygltf::Accessor accessor) {
    std::vector<glm_type> out;

    // Get number of components
    size_t n_components = 0;
    switch (accessor.type) {
    case TINYGLTF_TYPE_SCALAR:
        n_components = 1;
        break;
    case TINYGLTF_TYPE_VEC2:
        n_components = 2;
        break;
    case TINYGLTF_TYPE_VEC3:
        n_components = 3;
        break;
    case TINYGLTF_TYPE_VEC4:
        n_components = 4;
        break;
    default:
        printf("unknown gltf type %i!\n", accessor.type);
        break;
    }

    // Get component type and convert to glm type
    switch (accessor.componentType) {
    case TINYGLTF_COMPONENT_TYPE_FLOAT:
        out = pad_components_to_type<float, glm_type>((float*)pointer, n_components, glm_type::length(), accessor.count, accessor.normalized);
        break;
    case TINYGLTF_COMPONENT_TYPE_BYTE:
        out = pad_components_to_type<int8_t, glm_type>((int8_t*)pointer, n_components, glm_type::length(), accessor.count, accessor.normalized);
        break;
    case TINYGLTF_COMPONENT_TYPE_SHORT:
        out = pad_components_to_type<int16_t, glm_type>((int16_t*)pointer, n_components, glm_type::length(), accessor.count, accessor.normalized);
        break;
    case TINYGLTF_COMPONENT_TYPE_INT:
        out = pad_components_to_type<int32_t, glm_type>((int32_t*)pointer, n_components, glm_type::length(), accessor.count, accessor.normalized);
        break;
    case TINYGLTF_COMPONENT_TYPE_UNSIGNED_BYTE:
        out = pad_components_to_type<uint8_t, glm_type>((uint8_t*)pointer, n_components, glm_type::length(), accessor.count, accessor.normalized);
        break;
    case TINYGLTF_COMPONENT_TYPE_UNSIGNED_SHORT:
        out = pad_components_to_type<uint16_t, glm_type>((uint16_t*)pointer, n_components, glm_type::length(), accessor.count, accessor.normalized);
        break;
    case TINYGLTF_COMPONENT_TYPE_UNSIGNED_INT:
        out = pad_components_to_type<uint32_t, glm_type>((uint32_t*)pointer, n_components, glm_type::length(), accessor.count, accessor.normalized);
        break;
    default:
        printf("unknown gltf component type %i!\n", accessor.type);
    }
    return out;
}


void ModelResource::create_vertex_array(MeshBufferData& mesh_out, tinygltf::Primitive primitive_in, tinygltf::Model model, glm::mat4 trans_mat)
{
	std::vector<glm::vec3> position_pointer;
	std::vector<glm::vec3> normal_pointer;
	std::vector<glm::vec4> tangent_pointer;
    std::vector<glm::vec4> colour_pointer;
	std::vector<glm::vec2> texcoord_pointer;
	std::vector<int> indices;

	for (auto& attrib : primitive_in.attributes)
	{
		//Structure binding type beat but I'm too lazy to switch to C++17 lmao
		auto& name = attrib.first;
		auto& accessor_index = attrib.second;

		//Get accessor
		auto& accessor = model.accessors[accessor_index];

		//Get bufferview
		auto& bufferview_index = accessor.bufferView;
		auto& bufferview = model.bufferViews[bufferview_index];

		//Find location in buffer
		auto& buffer_base = model.buffers[bufferview.buffer].data;
		void* buffer_pointer = &buffer_base[bufferview.byteOffset];
		assert(bufferview.byteStride == 0 && "byte_stride is not zero!");

        printf("\nname: %s\n", name.c_str());

		if (name._Equal("POSITION"))
		{
			position_pointer = gltf_to_glm<glm::vec3>(buffer_pointer, accessor);
        }
        else if (name._Equal("NORMAL"))
        {
            normal_pointer = gltf_to_glm<glm::vec3>(buffer_pointer, accessor);
        }
        else if (name._Equal("TANGENT"))
        {
            tangent_pointer = gltf_to_glm<glm::vec4>(buffer_pointer, accessor);
        }
        else if (name._Equal("TEXCOORD_0"))
        {
            texcoord_pointer = gltf_to_glm<glm::vec2>(buffer_pointer, accessor);
        }
        else if (name._Equal("COLOR_0"))
        {
            // todo: add type checking here cuz it's not always a short vector 3 now is it
            colour_pointer = gltf_to_glm<glm::vec4>(buffer_pointer, accessor);
		}
	}

	//Find indices
	{
		//Get accessor
		auto& accessor = model.accessors[primitive_in.indices];

		//Get bufferview
		auto& bufferview_index = accessor.bufferView;
		auto& bufferview = model.bufferViews[bufferview_index];

		//Find location in buffer
		auto& buffer_base = model.buffers[bufferview.buffer].data;
		void* buffer_pointer = &buffer_base[bufferview.byteOffset];
		int buffer_length = accessor.count;
		indices.reserve(buffer_length);
		assert(bufferview.byteStride == 0 && "byte_stride is not zero!");

		if (accessor.componentType == TINYGLTF_COMPONENT_TYPE_UNSIGNED_SHORT)
		{
			auto* indices_raw = static_cast<uint16_t*>(buffer_pointer);
			for (int i = 0; i < buffer_length; i++)
			{
				indices.push_back(indices_raw[i]);
			}
		}
		if (accessor.componentType == TINYGLTF_COMPONENT_TYPE_UNSIGNED_INT)
		{
			auto* indices_raw = static_cast<uint32_t*>(buffer_pointer);
			for (int i = 0; i < buffer_length; i++)
			{
				indices.push_back(indices_raw[i]);
			}
		}
	}

	//Create vertex array
	{
		ResourceManager::get_allocator_instance()->curr_memory_chunk_label = "mesh loading - vertex buffers" ;
		mesh_out.verts = static_cast<Vertex*>(dynamic_allocate(sizeof(Vertex) * indices.size()));
		ResourceManager::get_allocator_instance()->curr_memory_chunk_label = "unknown";
		mesh_out.n_verts = 0;
		for (int index : indices)
		{
			Vertex vertex;
			if (!position_pointer.empty()) { vertex.position = trans_mat * glm::vec4(position_pointer[index], 1.0f); }
			if (!normal_pointer  .empty()) { vertex.normal   = glm::mat3(trans_mat) * (normal_pointer[index]); }
			if (!tangent_pointer .empty()) { vertex.tangent  = glm::mat3(trans_mat) * (glm::vec3(tangent_pointer[index])); }
            if (!colour_pointer.empty()) {
                vertex.colour = {
                    std::min(1.0f, powf(colour_pointer[index].x, 1.0f/2.2f)),
                    std::min(1.0f, powf(colour_pointer[index].y, 1.0f/2.2f)),
                    std::min(1.0f, powf(colour_pointer[index].z, 1.0f/2.2f)),
                };
            }
			if (!texcoord_pointer.empty()) { vertex.texcoord = texcoord_pointer[index]; }
			mesh_out.verts[mesh_out.n_verts++] = vertex;
		}
	}
}
