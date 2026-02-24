pub fn init_full(vregions: &Vec<MemRegion>, mmio_regions: &Vec<MemRegion>) -> (result: InitResult)
    requires
        forall|i: int| #![auto] 0 <= i < vregions.len() as int ==>
            vregions[i].spec_is_valid(),
        forall|i: int| #![auto] 0 <= i < mmio_regions.len() as int ==>
            mmio_regions[i].spec_is_valid(),
        forall|i: int| #![auto] 0 <= i < vregions.len() as int ==>
            vregions[i].start as int % INIT_PAGE_SIZE as int == 0
            && vregions[i].size as int % INIT_PAGE_SIZE as int == 0,
        forall|i: int| #![auto] 0 <= i < mmio_regions.len() as int ==>
            mmio_regions[i].start as int % INIT_PAGE_SIZE as int == 0
            && mmio_regions[i].size as int % INIT_PAGE_SIZE as int == 0,
        forall|i: int| #![auto] 0 <= i < vregions.len() as int ==>
            vregions[i].spec_end() <= INIT_MEMORY_SIZE as int,
        forall|i: int| #![auto] 0 <= i < mmio_regions.len() as int ==>
            mmio_regions[i].spec_end() <= INIT_MEMORY_SIZE as int,
    ensures
        result.spec_is_ok() ==> match result {
            InitResult::Ok { bases, mappings } => {
                // Base alignment.
                &&& forall|i: int| #![auto] 0 <= i < bases.len() as int ==>
                    bases[i] as int % INIT_PGTAB_ALIGNMENT as int == 0
                // Bases are strictly increasing.
                &&& forall|i: int, j: int|
                    #![trigger bases[i], bases[j]]
                    0 <= i < j < bases.len() as int ==>
                    (bases[i] as int) < (bases[j] as int)
                // Mapping correctness: paddr matches spec_init_paddr.
                &&& forall|k: int| #![auto] 0 <= k < mappings@.len() ==>
                    mappings@[k].paddr == spec_init_paddr(
                        mappings@[k].vaddr, mappings@[k].region_start, mappings@[k].is_mmio)
                // Identity mapping: non-MMIO pages have paddr == vaddr.
                &&& forall|k: int| #![auto] 0 <= k < mappings@.len() ==>
                    (!mappings@[k].is_mmio ==> mappings@[k].paddr == mappings@[k].vaddr)
                // Alignment: all mapped vaddrs are page-aligned.
                &&& forall|k: int| #![auto] 0 <= k < mappings@.len() ==>
                    mappings@[k].vaddr % INIT_PAGE_SIZE as int == 0
                // Alignment: all mapped paddrs are page-aligned.
                &&& forall|k: int| #![auto] 0 <= k < mappings@.len() ==>
                    mappings@[k].paddr % INIT_PAGE_SIZE as int == 0
                // No double-mapping: all mapped vaddrs are strictly increasing.
                &&& forall|i: int, j: int|
                    #![trigger mappings@[i], mappings@[j]]
                    0 <= i < j < mappings@.len() ==>
                    mappings@[i].vaddr < mappings@[j].vaddr
                // Permissions: all mappings have init-time permission attributes.
                &&& forall|k: int| #![auto] 0 <= k < mappings@.len() ==>
                    spec_has_init_permissions(mappings@[k])
            },
            _ => false,
        },
{
    let mut merged: Vec<MemRegion> = merge_regions(vregions, mmio_regions);
    sort_regions_by_start(&mut merged);
    init_checked(&merged)
}
