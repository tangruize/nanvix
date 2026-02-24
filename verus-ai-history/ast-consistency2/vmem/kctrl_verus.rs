    pub fn kctrl(&mut self, vaddr: usize, access: AccessPermission) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            spec_is_kernel_addr(vaddr as int),
            vaddr as int % PAGE_SIZE as int == 0,
        ensures
            self.inv(),
            self@.mapping_count == old(self)@.mapping_count,
    {
        unimplemented!()
    }
