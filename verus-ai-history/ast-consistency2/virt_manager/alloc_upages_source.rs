    pub fn alloc_upages(
        &mut self,
        vmem: &mut Vmem,
        mut vaddr: PageAligned<VirtualAddress>,
        nframes: usize,
        access: AccessPermission,
    ) -> Result<(), Error> {
        trace!("vaddr={:?}, nframes={}", vaddr, nframes);

        let physman: Rc<RefCell<PhysMemoryManager>> = self.physman.clone();

        let page_table_allocator = move || {
            let kframe: KernelFrame = match physman.try_borrow_mut() {
                Ok(mut physman) => physman.alloc_kernel_frame(true)?,
                Err(_) => {
                    let reason: &str = "failed to borrow physical memory manager";
                    error!("{reason}");
                    return Err(Error::new(ErrorCode::ResourceBusy, reason));
                },
            };
            let kpage: KernelPage = KernelPage::new(kframe);
            let pgtable_storage: PageTableStorage = PageTableStorage::KernelPage(kpage);
            let page_table: PageTable<PageTableStorage> = PageTable::new(pgtable_storage);
            Ok(page_table)
        };

        let uframes: Vec<UserFrame> = match self.physman.try_borrow_mut() {
            Ok(mut physman) => physman.alloc_many_user_frames(nframes)?,
            Err(_) => {
                let reason: &str = "failed to borrow physical memory manager";
                error!("{reason}");
                return Err(Error::new(ErrorCode::ResourceBusy, reason));
            },
        };

        // FIXME: check if range is not busy.

        for uframe in uframes {
            vmem.map(uframe, vaddr, access, &page_table_allocator)?;
            vaddr = PageAligned::from_raw_value(vaddr.into_raw_value() + mem::PAGE_SIZE)?;
        }

        Ok(())
    }
