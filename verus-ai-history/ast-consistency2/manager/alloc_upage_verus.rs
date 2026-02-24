    pub fn alloc_upage(
        &mut self,
        vmem: &mut Vmem,
        vaddr: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(vmem).inv(),
            old(self)@.has_upool_capacity(),
            old(vmem)@.has_mapping_capacity(),
            vaddr as int % PAGE_SIZE as int == 0,
            spec_is_user_addr(vaddr as int),
            !old(vmem)@.spec_is_mapped(vaddr as int),
        ensures
            self.inv(),
            vmem.inv(),
            result.is_ok() ==> {
                &&& vmem@.spec_is_mapped(vaddr as int)
                &&& vmem@.mapping_count == old(vmem)@.mapping_count + 1
            },
    {
        // Allocate user frame.
        let uframe: UserFrame = self.upool.alloc()?;

        // Get the frame address for mapping.
        let frame_addr: FrameAddress = uframe.address();

        proof {
            // Connect uframe alignment to frame_addr alignment.
            assert(uframe.spec_is_aligned());
            assert(frame_addr == uframe.spec_address());
            assert(frame_addr.spec_is_aligned());
            // Vmem is unchanged after upool.alloc(), so preconditions for map() still hold.
            assert(vmem.inv());
            assert(vmem@.has_mapping_capacity());
            assert(!vmem@.spec_is_mapped(vaddr as int));
        }

        // Map the frame to the virtual address.
        vmem.map(frame_addr, vaddr, access)?;

        proof {
            // vmem.map() postcondition gives us spec_is_mapped.
            assert(vmem@.spec_is_mapped(vaddr as int));
        }

        Ok(())
    }
