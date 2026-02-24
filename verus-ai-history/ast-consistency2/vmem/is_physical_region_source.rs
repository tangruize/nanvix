    pub fn is_physical_region(start: usize, size: usize) -> bool {
        // Reject zero-length regions.
        if size == 0 {
            return false;
        }

        // Check if the start and end addresses of the region lie within physical memory.
        match start.checked_add(size - 1) {
            Some(end) => start < MEMORY_SIZE && end < MEMORY_SIZE,
            None => false,
        }
    }
