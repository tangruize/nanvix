    pub unsafe fn get<'a>() -> &'a VirtMemoryManager {
        if let Some(ref mm) = MEMORY_MANAGER {
            mm
        } else {
            cold_path();
            panic!("the memory manager is not initialized");
        }
    }
