    fn find_user_frame(&self, vaddr: PageAligned<VirtualAddress>) -> Result<FrameAddress, Error> {
        let page_addr: PageAddress = PageAddress::new(vaddr);
        let pgtab_addr: PageTableAddress = PageTableAddress::new(PageTableAligned::from_raw_value(
            ::sys::mm::align_down(vaddr.into_raw_value(), PGTAB_ALIGNMENT),
        )?);

        // Look for the corresponding page table.
        for (lookup_pgtable_addr, page_table) in self.user_page_tables.iter() {
            // Found.
            if lookup_pgtable_addr == &pgtab_addr {
                // Look for the corresponding page.
                return page_table.lookup(page_addr);
            }
        }

        let reason: &str = "page not found";
        error!("{reason} (vaddr={vaddr:?})");
        Err(Error::new(ErrorCode::NoSuchEntry, reason))
    }
