    pub unsafe fn get_mut<'a>() -> &'a mut VirtMemoryManager {
        if let Some(ref mut mm) = MEMORY_MANAGER {
            mm
        } else {
            cold_path();
            panic!("the memory manager is not initialized");
        }
    }
