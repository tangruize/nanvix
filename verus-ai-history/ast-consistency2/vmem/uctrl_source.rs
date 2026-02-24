    pub fn uctrl(
        &mut self,
        vaddr: PageAligned<VirtualAddress>,
        access: AccessPermission,
    ) -> Result<(), Error> {
        // Check if the provided address lies outside the user space.
        if !Self::is_user_addr(vaddr.into_inner()) {
            let reason: &str = "address is not in user space";
            error!("{reason}");
            return Err(Error::new(ErrorCode::BadAddress, reason));
        }

        // Get corresponding page table.
        let page_table: &mut PageTable<PageTableStorage> = {
            let vaddr: PageTableAligned<VirtualAddress> = PageTableAligned::from_raw_value(
                ::sys::mm::align_down(vaddr.into_raw_value(), PGTAB_ALIGNMENT),
            )?;
            // Get the corresponding page directory entry.
            let pde: PageDirectoryEntry = match self.pgdir.read_pde(PageTableAddress::new(vaddr)) {
                Some(pde) => pde,
                None => {
                    let reason: &str = "failed to read page directory entry";
                    error!("{reason}");
                    return Err(Error::new(ErrorCode::TryAgain, reason));
                },
            };

            // Check if corresponding page table does not exist.
            if !pde.is_present() {
                let reason: &str = "page table not present";
                error!("{reason}");
                return Err(Error::new(ErrorCode::NoSuchEntry, reason));
            };

            self.lookup_page_table(&pde)?
        };

        let page_address: PageAddress = PageAddress::new(vaddr);

        // Change access permissions on the page.
        page_table.ctrl(false, page_address, access)?;

        Ok(())
    }
