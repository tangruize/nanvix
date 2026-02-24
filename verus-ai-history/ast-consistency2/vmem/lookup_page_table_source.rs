    fn lookup_page_table(
        &mut self,
        pde: &PageDirectoryEntry,
    ) -> Result<&mut PageTable<PageTableStorage>, Error> {
        // Check if corresponding page table does not exist.
        if !pde.is_present() {
            let reason: &str = "page table not present";
            error!("{reason:?} (pde={pde:?})");
            return Err(Error::new(ErrorCode::NoSuchEntry, reason));
        }

        // Get corresponding page table.
        let pgtab_addr: FrameAddress = FrameAddress::from_frame_number(pde.frame())?;

        // Find corresponding page table.
        let mut page_table: Option<&mut PageTable<PageTableStorage>> = None;
        for (_pgtable_vaddr, pt) in self.user_page_tables.iter_mut() {
            if pt.physical_address()? == pgtab_addr {
                page_table = Some(pt);
                break;
            }
        }

        match page_table {
            Some(pt) => Ok(pt),
            None => {
                let reason: &str = "page table not found";
                error!("{reason}");
                Err(Error::new(ErrorCode::NoSuchEntry, reason))
            },
        }
    }
