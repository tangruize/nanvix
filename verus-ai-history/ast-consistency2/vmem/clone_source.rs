    pub fn clone(from: &Vmem) -> Result<Vmem, Error> {
        // Create a clean page directory.
        let mut pgdir: PageDirectory = PageDirectory::new(PageDirectoryStorage::new());
        pgdir.clean();

        // Map and store root page tables.
        let mut kernel_page_tables: LinkedList<
            Rc<RefCell<(PageTableAddress, PageTable<PageTableStorage>)>>,
        > = LinkedList::new();
        for entry in from.kernel_page_tables.iter() {
            let page_table_address: FrameAddress = entry.borrow().1.physical_address()?;
            // FIXME: do not be so open about permissions.
            pgdir.map(entry.borrow().0, page_table_address, false, AccessPermission::RDWR)?;
            kernel_page_tables.push_back(entry.clone());
        }

        // Store root pages.
        let mut kernel_pages: LinkedList<Rc<RefCell<KernelPage>>> = LinkedList::new();
        for entry in from.kernel_pages.iter() {
            kernel_pages.push_back(entry.clone());
        }

        Ok(Self {
            pgdir,
            kernel_page_tables,
            kernel_pages,
            private_kernel_pages: LinkedList::new(),
            user_page_tables: LinkedList::new(),
        })
    }
