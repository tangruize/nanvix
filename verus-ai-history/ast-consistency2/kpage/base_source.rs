    pub fn base(&self) -> PageAddress {
        // TODO: rename this function to `page_address()`.
        PageAddress::new(
            self.kframe
                .base()
                .into_page_address()
                .into_virtual_address(),
        )
    }
