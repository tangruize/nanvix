pub fn get_page_paddr(vaddr: usize, region_start: usize, is_mmio: bool) -> (result: usize)
    requires
        vaddr as int % INIT_PAGE_SIZE as int == 0,
        region_start as int % INIT_PAGE_SIZE as int == 0,
    ensures
        !is_mmio ==> result == vaddr,
        result as int == spec_init_paddr(vaddr as int, region_start as int, is_mmio),
        result as int % INIT_PAGE_SIZE as int == 0,
{
    if is_mmio {
        // MMIO: use the region start address for physical address lookup.
        // This is external_body in the original (unsafe from_mmio_address).
        get_mmio_paddr(region_start)
    } else {
        // Non-MMIO: identity mapping.
        vaddr
    }
}
