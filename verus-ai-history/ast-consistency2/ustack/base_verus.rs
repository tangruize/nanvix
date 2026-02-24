    pub fn base(&self) -> (result: PageAlignedAddr)
        requires
            self.inv(),
        ensures
            result.inv(),
            result.spec_addr() == self.spec_base(),
    {
        PageAlignedAddr::from_raw_unchecked(self.base_addr)
    }
