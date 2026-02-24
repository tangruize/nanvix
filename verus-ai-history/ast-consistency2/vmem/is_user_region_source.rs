    pub fn is_user_region(start: VirtualAddress, size: usize) -> bool {
        // Reject zero-length regions.
        if size == 0 {
            return false;
        }

        // Check if the start and end addresses of the region lie in user space.
        match start.checked_add(size - 1) {
            Some(end) => Self::is_user_addr(start) && Self::is_user_addr(end),
            None => false,
        }
    }
