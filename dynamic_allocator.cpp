#include "dynamic_allocator.h"

struct MemoryManagerHeader
{
	uint32_t chunk_size_allocated; //4 byte aligned
	MemoryManagerHeader(const uint32_t size, bool allocated)
	{
		//Set chunk size
		chunk_size_allocated = size;

		//Align it to 4 bytes, to make room for the allocated flag
		while ((chunk_size_allocated & 0x03) != 0)
		{
			chunk_size_allocated++;
		}

		//Set allocated flag
		if (allocated)
		{
			chunk_size_allocated |= 0x01;
		}
	}
	uint32_t get_size_chunk() const
	{
		return (chunk_size_allocated ^ (chunk_size_allocated & 0x03));
	}
	void set_size_chunk(uint32_t size)
	{
		chunk_size_allocated = size;
	}
	bool is_free() const
	{
		return (chunk_size_allocated & 0x03) == 0x00;
	}
	void set_allocated(bool allocated)
	{
		chunk_size_allocated = ~(~chunk_size_allocated | 0x03);
		chunk_size_allocated += 1 * allocated;
	}
};

void DynamicAllocator::init(uint32_t size)
{
	block_start = malloc(size);
	block_size = size;
	static_cast<MemoryManagerHeader*>(block_start)->set_size_chunk(size);
	static_cast<MemoryManagerHeader*>(block_start)->set_allocated(false);
}

void* DynamicAllocator::allocate(uint32_t size, uint32_t align)
{
#ifdef NORMAL_ALLOC
	(void)align;
	return malloc(size);
#else
	//Alignment has to be a multiple of 4
	while (align % 4 != 0)
	{
		align++;
	}

	//Find first next empty chunk
	//Since everything is aligned to 4 bytes, I will create a uint32_t pointer to loop over the chunks
	uint32_t* memory_32 = static_cast<uint32_t*>(block_start);
	uint32_t size_required;
	uint32_t padding_bytes_required;
	uint32_t allocated_size_padded = static_cast<uint32_t>(size);
	if (allocated_size_padded % 4 != 0)
	{
		allocated_size_padded += 4 - (allocated_size_padded % 4);
	}

	while (true)
	{
		//Get size and allocation status of current chunk
		const uint32_t chunk_size = *memory_32 & ~(0x03);
		const uint32_t allocation_status = *memory_32 & 0x03;

		//We found a free memory chunk! Let's see if it can fit the requested size
		if (allocation_status == 0)
		{
			//Interpret the memory pointer as an integer. This is done so the alignment can be more easily calculated
			const intptr_t memory_pointer = reinterpret_cast<intptr_t>(memory_32);

			//We need a chunk size (header), a padding count and a chunk size (footer), which are all uint32_t
			constexpr unsigned int metadata_bytes_required = sizeof(uint32_t) * 3;

			//Then, we need to calculate how many padding bytes we will need to fit the alignment request
			const intptr_t memory_pointer_start = memory_pointer + 8;
			padding_bytes_required = static_cast<uint32_t>(memory_pointer_start % static_cast<intptr_t>(align));
			while ((memory_pointer_start + padding_bytes_required) % align != 0)
			{
				padding_bytes_required++;
			}

			//Let's check; do we have enough space for this
			size_required = metadata_bytes_required + padding_bytes_required + allocated_size_padded;

			//If we do have enough space in this chunk, break out of the loop and use this current memory pointer to allocate the memory
			if (chunk_size >= size_required)
			{
				break;
			}
		}

		//Otherwise, keep searching
		memory_32 += chunk_size / sizeof(uint32_t);

		//We should make sure we don't get out of bounds though
		const intptr_t current_byte_offset_into_memory = reinterpret_cast<intptr_t>(memory_32) - reinterpret_cast<intptr_t>(block_start);

		if (current_byte_offset_into_memory >= static_cast<int64_t>(block_size))
		{
			printf("[ERROR] Failed to allocate memory: Insufficient memory!");
			return nullptr;
		}
	}

	//We found a memory slot!

	//For debug purposes, label this memory
#ifdef DEBUG
	memory_labels[memory_32] = curr_memory_chunk_label;
#endif

	//First, we need to remember how big the original chunk was
	const uint32_t original_chunk_size = *memory_32 & ~(0x03);

	//Then, we save the new chunk size, and we set the "allocated" flag (0x01).
	*memory_32 = size_required | 0x01;
	memory_32++;

	//Move to right before where the data would start, and put the offset from header to data in there (this will be used in free() to determine where the header starts)
	memory_32 += (padding_bytes_required / 4);
	*memory_32 = padding_bytes_required + sizeof(uint32_t) * 2; //2x uint32_t; one for size, one for offset

	//Here we prepare the pointer for returning
	memory_32 += 1; //Move to start of data
	void* return_pointer = memory_32;

	//Move to the end of the data, set the footer, and move to the next chunk
	memory_32 += allocated_size_padded / 4;
	*memory_32 = size_required | 0x01;

	//Now let's add a header at the end of this with the remaining free size, if any, and make sure 
	const uint32_t remaining_free_size_in_this_chunk = original_chunk_size - size_required;

	if (remaining_free_size_in_this_chunk == 0)
	{
		return return_pointer;
	}

	//Set header of next chunk
	memory_32 += 1;
	*memory_32 = remaining_free_size_in_this_chunk;

	//Set footer of next cunk
	memory_32 += (remaining_free_size_in_this_chunk / 4) - 1;
	*memory_32 = remaining_free_size_in_this_chunk;

	//We're done!
	return return_pointer;
#endif
}

void DynamicAllocator::release(void* pointer)
{
#ifdef NORMAL_ALLOC
	return free(pointer);
#else
	if (pointer == nullptr)
		return;

	if (pointer < block_start || (intptr_t)pointer >= ((intptr_t)block_start + block_size))
	{
        printf("[ERROR] Attempted to release pointer at 0x%08X which is outside the range of the allocator, will skip this!");
		return;
	}

	//Get pointer to header using the offset right before the memory
	const uint32_t offset = static_cast<uint32_t*>(pointer)[-1];

	MemoryManagerHeader* header = reinterpret_cast<MemoryManagerHeader*>(static_cast<char*>(pointer) - reinterpret_cast<char*>(static_cast<intptr_t>(offset)));
#ifdef DEBUG
	memory_labels.erase(header);
#endif
	MemoryManagerHeader* header_center = header; //for use later

	//Keep track of the amount of free memory in this chunk
	intptr_t size_new_free_chunk = header->get_size_chunk();

	//Check if next memory chunk is also empty
	MemoryManagerHeader* next_header = header + (header->get_size_chunk() / sizeof(MemoryManagerHeader));

	//If there even is one
	const intptr_t int_memory_base = reinterpret_cast<intptr_t>(block_start);
	const intptr_t next_header_offset_from_base = reinterpret_cast<intptr_t>(next_header) - int_memory_base;
	if (next_header_offset_from_base < static_cast<int64_t>(block_size))
	{
		if (next_header->is_free())
		{
			size_new_free_chunk += next_header->get_size_chunk();
		}
	}

	//If the previous chunk is also empty, add the size to it, and move the pointer to the start of that chunk

	const MemoryManagerHeader* prev_header = header_center - 4 / sizeof(MemoryManagerHeader);
	const intptr_t prev_header_offset_from_base = reinterpret_cast<intptr_t>(prev_header) - int_memory_base;
	if (prev_header_offset_from_base >= 0)
	{
		if (prev_header->is_free())
		{
			size_new_free_chunk += prev_header->get_size_chunk();
			header = header_center - prev_header->get_size_chunk() / sizeof(MemoryManagerHeader);
		}
	}

	//Combine the chunks
	header->set_size_chunk(static_cast<uint32_t>(size_new_free_chunk));
	header->set_allocated(false);

	//Add copy the size_allocated to the end of the chunk
	uint32_t* chunk_end_size = reinterpret_cast<uint32_t*>(header + (size_new_free_chunk - 4) / sizeof(uint32_t));
	*chunk_end_size = static_cast<uint32_t>(size_new_free_chunk);
	
#endif
}

void* DynamicAllocator::reallocate(void* pointer, uint32_t size, uint32_t align)
{
#ifdef NORMAL_ALLOC
	(void)align;
	return realloc(pointer, size);
#else
	//In case that ptr is a null pointer, the function behaves like malloc, assigning a new block of size bytes and returning a pointer to its beginning.
	if (pointer == nullptr)
	{
		//According to the reallocate doc, if marker = 0 it should behave like an alloc instead. https://www.cplusplus.com/reference/cstdlib/reallocate/
		//Debug::logSysError("Realloc not possible... Using fallback.. Try to avoid this as much as possible!", "Memory");
		void* return_value = allocate(size, align);
		return return_value;
	}

	//Get pointer to header using the offset right before the memory
	uint32_t* marker_pointer = static_cast<uint32_t*>(pointer);
	const uint32_t offset = marker_pointer[-1];

	MemoryManagerHeader* header = reinterpret_cast<MemoryManagerHeader*>(static_cast<char*>(pointer) - reinterpret_cast<char*>(static_cast<intptr_t>(
		offset)));

	//Allocate new chunk
	void* new_memory_chunk = allocate(size, align);

	//Copy old data into new chunk. The content of the memory block is preserved up to the lesser of the new and old sizes https://www.cplusplus.com/reference/cstdlib/reallocate/
	int number_of_bytes_to_copy = std::min(static_cast<int>(header->get_size_chunk()), static_cast<int>(size));
	memcpy(new_memory_chunk, pointer, number_of_bytes_to_copy);

	//Free old chunk
	release(pointer);

	return new_memory_chunk;
#endif
}

void DynamicAllocator::debug_memory()
{
#ifdef DEBUG
	printf("------MEMORY-DEBUG------\n");
	MemoryManagerHeader* header = static_cast<MemoryManagerHeader*>(block_start);

	//Loop over every memory chunk
	while (reinterpret_cast<intptr_t>(header) < reinterpret_cast<intptr_t>(block_start) + static_cast<int>(block_size))
	{
		std::string free_occupied[]
		{
			"occupied",
			"free    ",
		};

		//If it's free, add number of bytes to the total
		const uint32_t size = header->get_size_chunk();
		printf("\tMemory Chunk: pointer: 0x%08x,\tsize: 0x%08x,\tstatus: %s,\tlabel: %s\n", header, size, free_occupied[(int)header->is_free()].c_str(), memory_labels[header].c_str());

		//Go to next chunk, the while loop condition takes care of breaking out
		header = header + (size / sizeof(MemoryManagerHeader));
	}
#endif
}

std::vector<MemoryChunk> DynamicAllocator::get_memory_chunk_list()
{
	std::vector<MemoryChunk> memory_chunks;
	for (auto a : memory_labels)
	{
		MemoryManagerHeader* header = static_cast<MemoryManagerHeader*>(a.first);
		memory_chunks.push_back({a.second, a.first, header->get_size_chunk(), header->is_free()});
	}
	return memory_chunks;
}
