pub fn get_nth_page_addr(region_start: usize, index: usize) -> (result: usize)
    requires
        region_start as int % INIT_PAGE_SIZE as int == 0,
        region_start as int + index as int * INIT_PAGE_SIZE as int <= usize::MAX as int,
    ensures
        result as int == spec_nth_page_addr(region_start as int, index as int),
        result as int % INIT_PAGE_SIZE as int == 0,
{
    region_start + index * INIT_PAGE_SIZE
}
