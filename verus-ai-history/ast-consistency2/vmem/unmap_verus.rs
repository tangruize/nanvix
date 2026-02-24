    pub fn unmap(&mut self, vaddr: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
            old(self)@.has_mappings(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                &&& spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
                &&& self@.mapping_count == old(self)@.mapping_count - 1
                // The returned value was a previously-mapped frame address.
                &&& old(self)@.spec_is_mapped(vaddr as int)
                // The returned frame address is frame-aligned.
                &&& result.unwrap() as int % FRAME_SIZE as int == 0
                // The returned frame address equals the one that was mapped.
                &&& result.unwrap() as int == old(self)@.spec_get_frame_addr(vaddr as int)
            },
            result.is_err() ==> {
                &&& self@.mapping_count == old(self)@.mapping_count
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

        // Capture old self for proving postcondition about spec_get_frame_addr.
        let ghost entry_self: Vmem = *self;

        // Find the mapping.
        let mut found_idx: usize = MAX_USER_PAGES;
        let mut frame_addr: usize = 0;
        let mut i: usize = 0;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count <= MAX_USER_PAGES,
                self.inv(),
                // Entry state is unchanged during search.
                self.mapping_count == entry_self.mapping_count,
                forall|j: int| #![auto] 0 <= j < self.mapping_count as int ==>
                    self.mappings[j as int] == entry_self.mappings[j as int],
                found_idx == MAX_USER_PAGES || found_idx < self.mapping_count,
                found_idx < MAX_USER_PAGES ==> {
                    &&& self.mappings[found_idx as int].valid
                    &&& self.mappings[found_idx as int].vaddr == vaddr
                    &&& frame_addr as int % FRAME_SIZE as int == 0
                    &&& frame_addr as int == self.mappings[found_idx as int].frame_addr as int
                },
            decreases self.mapping_count - i,
        {
            if self.mappings[i].valid && self.mappings[i].vaddr == vaddr {
                found_idx = i;
                frame_addr = self.mappings[i].frame_addr;
                break;
            }
            i = i + 1;
        }

        if found_idx == MAX_USER_PAGES {
            return Err(Error::new(ErrorCode::BadAddress, "page not mapped"));
        }

        proof {
            // entry_self has the same mappings as old(self) at function entry.
            // Connect to view-based specs for postcondition.
            assert(entry_self@.mappings[found_idx as int].spec_is_for_vaddr(vaddr as int));
            assert(entry_self@.spec_is_mapped(vaddr as int));
            assert(frame_addr as int == entry_self@.mappings[found_idx as int].frame_addr);
        }

        // Remove the mapping by swapping with the last entry.
        // Note: The original uses linked-list removal which preserves insertion order.
        // Swap-with-last changes ordering but is correct because the spec uses existential
        // quantifiers over the array — ordering is irrelevant for all invariants and
        // postconditions (uniqueness, mapping existence, count).
        let last_idx: usize = self.mapping_count - 1;
        if found_idx != last_idx {
            self.mappings[found_idx] = self.mappings[last_idx];
        }
        self.mappings[last_idx] = PageMapping {
            vaddr: 0,
            frame_addr: 0,
            valid: false,
        };
        self.mapping_count = self.mapping_count - 1;

        Ok(frame_addr)
    }
