pub fn init_checked(regions: &Vec<MemRegion>) -> (result: InitResult)
    requires
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].spec_is_valid(),
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].start as int % INIT_PAGE_SIZE as int == 0
            && regions[i].size as int % INIT_PAGE_SIZE as int == 0,
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].spec_end() <= INIT_MEMORY_SIZE as int,
    ensures
        // On success, all init postconditions hold.
        result.spec_is_ok() ==> match result {
            InitResult::Ok { bases, mappings } => {
                &&& forall|i: int| #![auto] 0 <= i < bases.len() as int ==>
                    bases[i] as int % INIT_PGTAB_ALIGNMENT as int == 0
                &&& forall|i: int, j: int|
                    #![trigger bases[i], bases[j]]
                    0 <= i < j < bases.len() as int ==>
                    (bases[i] as int) < (bases[j] as int)
                &&& mappings@.len() == spec_total_pages(regions@, regions.len() as int)
                &&& forall|k: int| #![auto] 0 <= k < mappings@.len() ==>
                    mappings@[k].paddr == spec_init_paddr(
                        mappings@[k].vaddr, mappings@[k].region_start, mappings@[k].is_mmio)
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
    if validate_regions(regions) {
        let (bases, mappings): (Vec<usize>, Ghost<Seq<PageMapping>>) = init(regions);
        InitResult::Ok { bases, mappings }
    } else {
        InitResult::OverlapError
    }
}
