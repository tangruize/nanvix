    fn is_kernel_region(start: usize, size: usize) -> (result: bool)
        ensures
            result ==> spec_is_kernel_region(start as int, size as int),
            result ==> size > 0,
            result ==> spec_is_kernel_addr(start as int),
    {
        // Reject zero-length regions.
        if size == 0 {
            return false;
        }

        // Check for overflow.
        let end_opt: Option<usize> = start.checked_add(size - 1);
        match end_opt {
            Some(end) => Self::is_kernel_addr(start) && Self::is_kernel_addr(end),
            None => false,
        }
    }
