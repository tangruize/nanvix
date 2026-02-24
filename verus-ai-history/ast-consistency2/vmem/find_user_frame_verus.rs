    fn find_user_frame(&self, vaddr: usize) -> (result: Result<usize, Error>)
        requires
            self.inv(),
        ensures
            result.is_ok() ==> {
                &&& spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
            },
    {
        // Check if address is in user space.
        if !Self::is_user_addr(vaddr) {
            return Err(Error::new(ErrorCode::BadAddress, "address is not in user space"));
        }

        // Check if address is page-aligned.
        if vaddr % PAGE_SIZE != 0 {
            return Err(Error::new(ErrorCode::BadAddress, "address is not page-aligned"));
        }

        // Find the mapping.
        let mut i: usize = 0;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count <= MAX_USER_PAGES,
                self.inv(),
            decreases self.mapping_count - i,
        {
            if self.mappings[i].valid && self.mappings[i].vaddr == vaddr {
                return Ok(self.mappings[i].frame_addr);
            }
            i = i + 1;
        }

        Err(Error::new(ErrorCode::BadAddress, "page not found"))
    }
