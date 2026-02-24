    pub fn init(
        kernel_pages: LinkedList<KernelPage>,
        kernel_page_tables: LinkedList<(PageTableAddress, PageTable<PageTableStorage>)>,
        physman: PhysMemoryManager,
    ) -> Result<(Vmem, VirtMemoryManager), Error> {
        // Check if the memory manager is already initialized.
        if unlikely(unsafe { MEMORY_MANAGER.is_some() }) {
            panic!("memory manager was already initialized");
        }

        let (root, manager): (Vmem, VirtMemoryManager) =
            VirtMemoryManager::new(kernel_pages, kernel_page_tables, physman)?;

        // SAFETY: This happens during kernel initialization and no other threads are running.
        unsafe {
            MEMORY_MANAGER = Some(manager.clone());
        }
        Ok((root, manager))
    }
