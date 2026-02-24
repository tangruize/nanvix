    fn alloc_range(&mut self, count: usize) -> Result<Vec<FrameAddress>, Error> {
        // Attempt to allocate a range of pages.
        let index: usize = match self.bitmap.alloc_range(count) {
            Ok(index) => index,
            Err(error) => {
                error!("{error:?} (count={count})");
                return Err(error);
            },
        };

        // Create a vector of page-aligned addresses.
        let base_addr: usize = self.region.start().into_raw_value() + index * mem::PAGE_SIZE;
        let mut pages: Vec<FrameAddress> = Vec::new();
        for i in 0..count {
            let addr: usize = base_addr + i * mem::PAGE_SIZE;
            let page: FrameAddress = FrameAddress::new(PageAligned::from_address(
                PhysicalAddress::from_raw_value(addr)?,
            )?);
            pages.push(page);
        }

        Ok(pages)
    }
