    pub fn uctrl(&mut self, vaddr: usize, access: AccessPermission) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                &&& spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
            },
            // Permission changes don't affect the mappings.
            self@.mapping_count == old(self)@.mapping_count,
    {
        // Check if address is in user space.
        if !Self::is_user_addr(vaddr) {
            return Err(Error::new(ErrorCode::BadAddress, "address is not in user space"));
        }

        // Check if address is page-aligned.
        if vaddr % PAGE_SIZE != 0 {
            return Err(Error::new(ErrorCode::BadAddress, "address is not page-aligned"));
        }

        // Check if page is mapped.
        let mut found: bool = false;
        let mut i: usize = 0;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count <= MAX_USER_PAGES,
                self.inv(),
                found ==> exists|j: int|
                    #![trigger self.mappings[j]]
                    0 <= j < self.mapping_count as int &&
                    self.mappings[j].valid &&
                    self.mappings[j].vaddr == vaddr,
            decreases self.mapping_count - i,
        {
            if self.mappings[i].valid && self.mappings[i].vaddr == vaddr {
                found = true;
                break;
            }
            i = i + 1;
        }

        if !found {
            return Err(Error::new(ErrorCode::BadAddress, "page not mapped"));
        }

        // In a real implementation, we would update the page table entry permissions.
        Ok(())
    }
