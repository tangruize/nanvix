pub fn get_mmio_paddr(region_start: usize) -> (result: usize)
    requires
        region_start as int % INIT_PAGE_SIZE as int == 0,
    ensures
        result as int == spec_mmio_paddr(region_start as int),
        result as int % INIT_PAGE_SIZE as int == 0,
{
    unimplemented!()
}
