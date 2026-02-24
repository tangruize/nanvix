    pub fn top(&self) -> PageAligned<VirtualAddress> {
        PageAligned::from_raw_value(self.base.into_raw_value() + self.size()).unwrap()
    }
