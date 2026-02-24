    fn is_kernel_region(start: VirtualAddress, size: usize) -> bool {
        // Reject zero-length regions.
        if size == 0 {
            return false;
        }

        // Check if the start and end addresses of the region lie in kernel space.
        match start.checked_add(size - 1) {
            Some(end) => Self::is_kernel_addr(start) && Self::is_kernel_addr(end),
            None => false,
        }
    }
