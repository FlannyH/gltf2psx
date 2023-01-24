#pragma once
#define dynamic_allocate ResourceManager::get_allocator_instance()->allocate
#define dynamic_reallocate ResourceManager::get_allocator_instance()->reallocate
#define dynamic_free ResourceManager::get_allocator_instance()->release