pub struct Vmem {
    /// Underlying page directory.
    pgdir: PageDirectory,
    /// List of kernel page tables.
    kernel_page_tables: LinkedList<Rc<RefCell<(PageTableAddress, PageTable<PageTableStorage>)>>>,
    /// List of kernel pages mapped in the virtual address space.
    /// NOTE: this currently excludes kernel pages that are identity mapped.
    kernel_pages: LinkedList<Rc<RefCell<KernelPage>>>,
    /// List of private kernel pages.
    private_kernel_pages: LinkedList<KernelPage>,
    /// List of user page tables.
    user_page_tables: LinkedList<(PageTableAddress, PageTable<PageTableStorage>)>,
}
