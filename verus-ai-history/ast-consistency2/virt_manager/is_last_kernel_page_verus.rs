pub fn is_last_kernel_page(vaddr: usize) -> (result: bool)
    requires
        INIT_MEMORY_SIZE >= INIT_PAGE_SIZE,
    ensures
        result == (vaddr == INIT_MEMORY_SIZE - INIT_PAGE_SIZE),
{
    vaddr == INIT_MEMORY_SIZE - INIT_PAGE_SIZE
}
