    pub fn alloc_upage(
        &mut self,
        vmem: &mut Vmem,
        vaddr: PageAligned<VirtualAddress>,
        access: AccessPermission,
        clear: bool,
    ) -> Result<(), Error> {
        let uframe: UserFrame = match self.physman.try_borrow_mut() {
            Ok(mut physman) => physman.alloc_user_frame()?,
            Err(_) => {
                let reason: &str = "failed to borrow physical memory manager";
                error!("{reason}");
                return Err(Error::new(ErrorCode::ResourceBusy, reason));
            },
        };

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

        vmem.map(uframe, vaddr, access, &page_table_allocator)?;

        // Check if the page should be cleared.
        if clear {
            // Safety: `vaddr` points to a valid memory location.
            vmem.memset(vaddr, 0)?;
        }

        Ok(())
    }
