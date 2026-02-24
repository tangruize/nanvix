    pub fn map<T: Fn() -> Result<PageTable<PageTableStorage>, Error>>(
        &mut self,
        uframe: UserFrame,
        vaddr: PageAligned<VirtualAddress>,
        access: AccessPermission,
        page_table_allocator: T,
    ) -> Result<(), Error> {
        // Check if the provided address lies outside the user space.
        if !Self::is_user_addr(vaddr.into_inner()) {
            let reason: &str = "address is not in user space";
            error!("{reason} (uframe={uframe:?}, vaddr={vaddr:?}, access={access:?})",);
            return Err(Error::new(ErrorCode::BadAddress, reason));
        }

        // Get corresponding page table.
        let page_table: &mut PageTable<PageTableStorage> = {
            let vaddr: PageTableAligned<VirtualAddress> = PageTableAligned::from_raw_value(
                ::sys::mm::align_down(vaddr.into_raw_value(), PGTAB_ALIGNMENT),
            )?;
            let pgtable_vaddr: PageTableAddress = PageTableAddress::new(vaddr);
            // Get the corresponding page directory entry.
            let mut pde: PageDirectoryEntry = match self.pgdir.read_pde(pgtable_vaddr) {
                Some(pde) => pde,
                None => {
                    let reason: &str = "failed to read page directory entry";
                    error!("{reason}");
                    return Err(Error::new(ErrorCode::TryAgain, reason));
                },
            };

            // Get corresponding page table.
            // Check if corresponding page table does not exist.
            if !pde.is_present() {
                let page_table: PageTable<PageTableStorage> = page_table_allocator()?;

                let page_table_address: FrameAddress = page_table.physical_address()?;
                // FIXME: do not be so open about permissions.
                self.pgdir
                    .map(pgtable_vaddr, page_table_address, false, AccessPermission::RDWR)?;

                //===================================================================
                // NOTE: if we fail beyond this point we should unmap the page table.
                //===================================================================

                self.user_page_tables.push_back((pgtable_vaddr, page_table));

                // Get the corresponding page directory entry.
                pde = match self.pgdir.read_pde(PageTableAddress::new(vaddr)) {
                    Some(pde) => pde,
                    None => unreachable!("failed to read page directory entry"),
                };
            };

            self.lookup_page_table(&pde)?
        };

        // Map the page to the target virtual address space.
        page_table.map(PageAddress::new(vaddr), uframe.address(), false, false, true, access)?;

        //=============================================================
        // NOTE: if we fail beyond this point we should unmap the page.
        //=============================================================

        Ok(())
    }
