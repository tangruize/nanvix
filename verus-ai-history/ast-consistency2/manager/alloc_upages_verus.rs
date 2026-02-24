    pub fn alloc_upages(
        &mut self,
        vmem: &mut Vmem,
        vaddr: usize,
        nframes: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(vmem).inv(),
            nframes > 0,
            old(self)@.has_upool_capacity_for(nframes as int),
            old(vmem)@.mapping_count + nframes as int <= MAX_USER_PAGES as int,
            vaddr as int % PAGE_SIZE as int == 0,
            spec_is_user_addr(vaddr as int),
            // All target addresses are in user space and not already mapped.
            forall|i: int|
                #![trigger spec_is_user_addr(vaddr as int + i * PAGE_SIZE as int)]
                0 <= i < nframes as int ==>
                    spec_is_user_addr(vaddr as int + i * PAGE_SIZE as int) &&
                    !old(vmem)@.spec_is_mapped(vaddr as int + i * PAGE_SIZE as int),
        ensures
            self.inv(),
            vmem.inv(),
            result.is_ok() ==> {
                &&& self@.upool_free_count == old(self)@.upool_free_count - nframes as int
                &&& vmem@.mapping_count == old(vmem)@.mapping_count + nframes as int
            },
            result.is_err() ==> vmem@.mapping_count == old(vmem)@.mapping_count,
    {
        unimplemented!()
    }
