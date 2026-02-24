    pub fn ctrl_upage(
        &self,
        vmem: &mut Vmem,
        vaddr: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            self.inv(),
            old(vmem).inv(),
            vaddr as int % PAGE_SIZE as int == 0,
            spec_is_user_addr(vaddr as int),
            old(vmem)@.spec_is_mapped(vaddr as int),
        ensures
            self.inv(),
            vmem.inv(),
            vmem@.mapping_count == old(vmem)@.mapping_count,
    {
        vmem.uctrl(vaddr, access)
    }
