    pub fn map(
        &mut self,
        frame_addr: FrameAddress,
        vaddr: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(self)@.has_mapping_capacity(),
            frame_addr.spec_is_aligned(),
            // Liveness: require valid parameters for guaranteed success.
            spec_is_user_addr(vaddr as int),
            vaddr as int % PAGE_SIZE as int == 0,
            !old(self)@.spec_is_mapped(vaddr as int),
        ensures
            self.inv(),
            // LIVENESS: With all preconditions met, map always succeeds.
            result.is_ok(),
            result.is_ok() ==> {
                &&& self@.spec_is_mapped(vaddr as int)
                &&& self@.mapping_count == old(self)@.mapping_count + 1
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

        // Check if we have space.
        if self.mapping_count >= MAX_USER_PAGES {
            return Err(Error::new(ErrorCode::OutOfMemory, "no mapping slots available"));
        }

        // Check if already mapped by scanning existing entries.
        // With precondition !old(self)@.spec_is_mapped(vaddr), this loop won't find a match.
        let mut i: usize = 0;
        let ghost old_mapping_count: usize = self.mapping_count;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count < MAX_USER_PAGES,
                self.inv(),
                self.mapping_count == old_mapping_count,
                // No modification to mappings during scan.
                !self@.spec_is_mapped(vaddr as int),
                forall|j: int| #![auto] 0 <= j < i as int ==>
                    !(self.mappings[j as int].valid && self.mappings[j as int].vaddr == vaddr),
            decreases self.mapping_count - i,
        {
            if self.mappings[i].valid && self.mappings[i].vaddr == vaddr {
                proof {
                    // This branch contradicts the invariant !self@.spec_is_mapped(vaddr).
                    assert(self@.mappings[i as int].spec_is_for_vaddr(vaddr as int));
                    assert(self@.spec_is_mapped(vaddr as int));
                    assert(false);
                }
                return Err(Error::new(ErrorCode::ResourceBusy, "page already mapped"));
            }
            i = i + 1;
        }

        // Add the new mapping.
        let slot: usize = self.mapping_count;
        proof {
            frame_addr.lemma_inv_from_aligned();
        }
        self.mappings[slot] = PageMapping {
            vaddr: vaddr,
            frame_addr: frame_addr.into_raw_value(),
            valid: true,
        };
        self.mapping_count = self.mapping_count + 1;

        // Help Verus see that the new mapping satisfies spec_is_mapped.
        proof {
            let idx: int = slot as int;
            let count: int = self.mapping_count as int;
            assert(self.mappings[idx].valid);
            assert(self.mappings[idx].vaddr as int == vaddr as int);
            assert(self@.mappings[idx].spec_is_for_vaddr(vaddr as int));
            assert(idx >= 0);
            assert(idx < count);
        }

        Ok(())
    }
