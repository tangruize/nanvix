    fn new(
        kernel_pages: LinkedList<KernelPage>,
        kernel_page_tables: LinkedList<(PageTableAddress, PageTable<PageTableStorage>)>,
        physman: PhysMemoryManager,
    ) -> Result<(Vmem, Self), Error> {
        let root: Vmem = Vmem::new(kernel_pages, kernel_page_tables)?;

        // Load root root address space.
        root.load()?;

        Ok((
            root,
            Self {
                physman: Rc::new(RefCell::new(physman)),
            },
        ))
    }
