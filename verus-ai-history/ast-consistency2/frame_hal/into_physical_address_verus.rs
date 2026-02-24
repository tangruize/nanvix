    pub fn into_physical_address(self) -> (result: PageAlignedPhysAddr)
        requires self.inv(),
        ensures
            result.inv(),
            result.spec_raw_value() == self.spec_raw_value(),
    {
        PageAlignedPhysAddr { raw_addr: self.raw_addr }
    }
