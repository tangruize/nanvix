pub fn compute_region_page_count(size: usize) -> (result: usize)
    requires
        size > 0,
        size as int % INIT_PAGE_SIZE as int == 0,
    ensures
        result as int == spec_page_count(size as int),
        result > 0,
{
    size / INIT_PAGE_SIZE
}
