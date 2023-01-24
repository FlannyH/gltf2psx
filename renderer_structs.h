#pragma once
#include <GL/glcorearb.h>
#include <glfw/glfw3.h>
#include <glm/matrix.hpp>
#include <glm/vec2.hpp>
#include <glm/vec3.hpp>
#include "resource_handler_structs.h"

#ifdef DIRECTX12

#include <wrl.h>
#include <d3d12.h>
#include <d3d12sdklayers.h>
#include <dxgi1_6.h>

using Microsoft::WRL::ComPtr;
#endif

struct Vertex
{
	glm::vec3 position{ 0,0,0 };
	glm::vec3 normal { 0,1,0 };
	glm::vec3 tangent { 0,0,1 };
	glm::vec3 colour { 1,1,1 };
	glm::vec2 texcoord { 0,0 };
};


struct MeshGPU
{
    int vert_count = 0;
#ifdef OPENGL
	GLuint vao = 0;
	GLuint vbo = 0;
#else if DIRECTX12
    ComPtr<ID3D12Resource> vertex_buffer;
    D3D12_VERTEX_BUFFER_VIEW vertex_buffer_view;
    ComPtr<ID3D12Resource> index_buffer;
    D3D12_INDEX_BUFFER_VIEW index_buffer_view;
#endif
};


struct TextureGPU
{
	GLuint handle = 0;
};

struct ShaderGPU
{
	GLuint handle = 0;
};

struct MaterialGPU
{
	TextureGPU tex_col;
	TextureGPU tex_nrm;
	TextureGPU tex_rgh;
	TextureGPU tex_mtl;
	TextureGPU tex_emm;
	glm::vec4 mul_col;
	glm::vec3 mul_nrm;
	float mul_rgh;
	float mul_mtl;
	glm::vec3 mul_emm;
	glm::vec2 mul_tex;
};

struct ModelGPU
{
	MeshGPU* meshes;
	MaterialGPU* materials;
	int n_meshes;
	int n_materials;
};

struct MeshBufferData
{
	Vertex* verts;
	int n_verts;
};

struct FrameBufferData
{
	GLuint fbo;
	TextureGPU fb_col;
	TextureGPU fb_depth;
	ShaderGPU fb_shader;
	MeshGPU fb_quad;
};

struct RenderContextData
{
	GLFWwindow* window;
	glm::ivec2 resolution{ 1280,720 };
	bool fullscreen = false;
#ifdef DIRECTX12
    inline static constexpr int backbuffer_count = 2;
    HWND hwnd{};
    ComPtr<IDXGIFactory4> factory;
#ifdef _DEBUG
    ComPtr<ID3D12Debug1> debug_interface;
    ComPtr<ID3D12DebugDevice> device_debug;
    ComPtr<ID3D12Debug> debug_layer;
#endif
    ComPtr<ID3D12Device> device;
    ComPtr<ID3D12DescriptorHeap> rtv_heap;
    ComPtr<ID3D12CommandQueue> command_queue;
    ComPtr<IDXGISwapChain3> swapchain;
    ComPtr<ID3D12Resource> render_targets[backbuffer_count];
    ComPtr<ID3D12Fence> fences[backbuffer_count];
    D3D12_DESCRIPTOR_HEAP_DESC rtv_heap_desc;
    DXGI_SWAP_CHAIN_DESC1 swapchain_desc{};
    UINT frame_index;
    UINT rtv_desc_size;
    UINT dxgi_factory_flags = 0;
#endif
};

enum class ShaderType
{
	vertex,
	pixel,
	geometry,
	compute,
};

struct CameraDataConstantBuffer
{
	glm::mat4 model_matrix = glm::mat4(1.0f);
	glm::mat4 view_matrix = glm::mat4(1.0f);
	glm::mat4 proj_matrix = glm::mat4(1.0f);
	glm::vec3 view_pos;
};

struct MaterialDataConstantBuffer
{
	glm::vec3 mul_col;
	glm::vec3 mul_nrm;
	glm::vec3 mul_rgh;
	glm::vec3 mul_mtl;
	glm::vec3 mul_emm;
	glm::vec2 mul_tex;
};

enum class ConstantBufferType
{
	camera_data,
	material_data
};

struct ConstantBufferGPU
{
#ifdef OPENGL
	GLuint handle = 0;
#else if DIRECTX12

    ComPtr<ID3D12DescriptorHeap> heap;
    ComPtr<ID3D12Resource> resource;
    void* data;
    size_t size;
    D3D12_CONSTANT_BUFFER_VIEW_DESC view_desc;
    D3D12_RANGE range;
#endif
};

struct ModelRenderComponent
{
	ResourceHandle model;
};

struct MeshRenderData
{
	MeshGPU mesh;
	MaterialGPU material;
	glm::mat4 model_matrix;
};