    pub fn is_physical_region(start: usize, size: usize) -> (result: bool)
        ensures
            result ==> spec_is_physical_region(start as int, size as int),
            result ==> size > 0,
            result ==> start < MEMORY_SIZE,
    {
        // Reject zero-length regions.
        if size == 0 {
            return false;
        }

        // Check for overflow.
        let end_opt: Option<usize> = start.checked_add(size - 1);
        match end_opt {
            Some(end) => start < MEMORY_SIZE && end < MEMORY_SIZE,
            None => false,
        }
    }
