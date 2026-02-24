    pub fn into_page_address(self) -> PageAddress {
        PageAddress::new(PageAligned::into_virtual_address(self.0))
    }
