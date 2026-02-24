    pub fn is_user_region(start: usize, size: usize) -> (result: bool)
        ensures
            result ==> spec_is_user_region(start as int, size as int),
            result ==> size > 0,
            result ==> spec_is_user_addr(start as int),
    {
        // Reject zero-length regions.
        if size == 0 {
            return false;
        }

        // Check for overflow.
        let end_opt: Option<usize> = start.checked_add(size - 1);
        match end_opt {
            Some(end) => Self::is_user_addr(start) && Self::is_user_addr(end),
            None => false,
        }
    }
