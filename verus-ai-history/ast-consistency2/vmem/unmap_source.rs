    pub fn unmap(&mut self, vaddr: PageAligned<VirtualAddress>) -> Result<UserFrame, Error> {
        // Check if the provided address lies outside the user space.
        if !Self::is_user_addr(vaddr.into_inner()) {
            let reason: &str = "address is not in user space";
            error!("{reason}");
            return Err(Error::new(ErrorCode::BadAddress, reason));
        }

        // Find the corresponding frame address.
        let frame_address: FrameAddress = self.find_user_frame(vaddr)?;

        let (pgtable_vaddr, unmap_pgtable): (PageTableAddress, bool) = {
            // Get corresponding page table.
            let (pgtable_vaddr, page_table): (PageTableAddress, &mut PageTable<PageTableStorage>) = {
                let vaddr: PageTableAligned<VirtualAddress> = PageTableAligned::from_raw_value(
                    ::sys::mm::align_down(vaddr.into_raw_value(), PGTAB_ALIGNMENT),
                )?;
                let pgtable_vaddr: PageTableAddress = PageTableAddress::new(vaddr);
                // Get the corresponding page directory entry.
                let pde: PageDirectoryEntry = match self.pgdir.read_pde(pgtable_vaddr) {
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
                    let reason: &str = "page table not present";
                    error!("{reason}");
                    return Err(Error::new(ErrorCode::NoSuchEntry, reason));
                };

                (pgtable_vaddr, self.lookup_page_table(&pde)?)
            };

            let page_address: PageAddress = PageAddress::new(vaddr);

            // Check if frame address matches what we expect.
            if page_table.lookup(page_address)? != frame_address {
                // The following statement should not be reachable because after mapping user frame we
                // must have added it to the list of user pages.
                unreachable!("frame address must match what we expect");
            }

            // Unmap the page from the target virtual address space.
            page_table.unmap(page_address)?;

            (pgtable_vaddr, page_table.nmapped() == 0)
        };

        //====================================================================================
        // NOTE: if we fail beyond this point and we want to recover we should remap the page.
        //====================================================================================

        if unmap_pgtable {
            // Remove page table from the list of user page tables.
            let at = self
                .user_page_tables
                .iter()
                .position(|(addr, _)| addr == &pgtable_vaddr)
                .expect("page table must be in the list of user page tables");

            let (_pgtable_addr, _page_table) = self.user_page_tables.remove(at);

            self.pgdir.unmap(pgtable_vaddr)?;
        }

        Ok(UserFrame::new(frame_address))
    }
