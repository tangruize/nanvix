    fn base(&self) -> PageAligned<VirtualAddress> {
        PageAligned::from_raw_value(self.kpages[0].base().into_raw_value()).unwrap()
    }
