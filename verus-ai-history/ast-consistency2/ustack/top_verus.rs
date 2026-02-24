    pub fn top(&self) -> (result: PageAlignedAddr)
        requires
            self.inv(),
        ensures
            result.inv(),
            result.spec_addr() == self.spec_top(),
    {
        PageAlignedAddr::from_raw_unchecked(self.base_addr + USER_STACK_SIZE)
    }
