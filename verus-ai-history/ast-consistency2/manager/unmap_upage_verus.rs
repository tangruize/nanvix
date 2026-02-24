    pub fn unmap_upage(
        &mut self,
        vmem: &mut Vmem,
        vaddr: usize,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(vmem).inv(),
            old(vmem)@.has_mappings(),
            vaddr as int % PAGE_SIZE as int == 0,
            spec_is_user_addr(vaddr as int),
            old(vmem)@.spec_is_mapped(vaddr as int),
            // The frame backing this mapping was allocated from the upool.
            // This is satisfied when the page was allocated via alloc_upage().
            old(self).spec_uframe_is_allocated(old(vmem)@.spec_get_frame_addr(vaddr as int)),
        ensures
            self.inv(),
            vmem.inv(),
            result.is_ok() ==> {
                &&& vmem@.mapping_count == old(vmem)@.mapping_count - 1
            },
    {
        // Unmap the page. Returns the frame address.
        let frame_addr: usize = vmem.unmap(vaddr)?;

        proof {
            // The returned frame_addr is aligned (from vmem.unmap postcondition).
            assert(frame_addr as int % FRAME_SIZE as int == 0);
        }

        // Free the frame back to the user pool.
        self.upool.free_by_addr(frame_addr)?;

        Ok(())
    }
