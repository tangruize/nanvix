pub fn page_table_map_page(vaddr: usize, paddr: usize)
    requires
        vaddr as int % INIT_PAGE_SIZE as int == 0,
        paddr as int % INIT_PAGE_SIZE as int == 0,
{
    // HAL-level unsafe PTE write - intentionally unimplemented in model.
}
