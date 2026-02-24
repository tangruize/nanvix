pub fn merge_regions(vregions: &Vec<MemRegion>, mmio_regions: &Vec<MemRegion>) -> (result: Vec<MemRegion>)
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
        result.len() == vregions.len() + mmio_regions.len(),
        forall|i: int| #![auto] 0 <= i < result.len() as int ==>
            result[i].spec_is_valid(),
        forall|i: int| #![auto] 0 <= i < result.len() as int ==>
            result[i].start as int % INIT_PAGE_SIZE as int == 0
            && result[i].size as int % INIT_PAGE_SIZE as int == 0,
        forall|i: int| #![auto] 0 <= i < result.len() as int ==>
            result[i].spec_end() <= INIT_MEMORY_SIZE as int,
{
    let mut merged: Vec<MemRegion> = Vec::new();
    let mut i: usize = 0;
    while i < vregions.len()
        invariant
            0 <= i <= vregions.len(),
            merged.len() == i,
            forall|k: int| #![auto] 0 <= k < merged.len() as int ==>
                merged[k].spec_is_valid(),
            forall|k: int| #![auto] 0 <= k < merged.len() as int ==>
                merged[k].start as int % INIT_PAGE_SIZE as int == 0
                && merged[k].size as int % INIT_PAGE_SIZE as int == 0,
            forall|k: int| #![auto] 0 <= k < merged.len() as int ==>
                merged[k].spec_end() <= INIT_MEMORY_SIZE as int,
            forall|k: int| #![auto] 0 <= k < vregions.len() as int ==>
                vregions[k].spec_is_valid(),
            forall|k: int| #![auto] 0 <= k < vregions.len() as int ==>
                vregions[k].start as int % INIT_PAGE_SIZE as int == 0
                && vregions[k].size as int % INIT_PAGE_SIZE as int == 0,
            forall|k: int| #![auto] 0 <= k < vregions.len() as int ==>
                vregions[k].spec_end() <= INIT_MEMORY_SIZE as int,
        decreases vregions.len() - i,
    {
        merged.push(MemRegion { start: vregions[i].start, size: vregions[i].size, is_mmio: vregions[i].is_mmio });
        i = i + 1;
    }
    let mut j: usize = 0;
    while j < mmio_regions.len()
        invariant
            0 <= j <= mmio_regions.len(),
            merged.len() == vregions.len() + j,
            forall|k: int| #![auto] 0 <= k < merged.len() as int ==>
                merged[k].spec_is_valid(),
            forall|k: int| #![auto] 0 <= k < merged.len() as int ==>
                merged[k].start as int % INIT_PAGE_SIZE as int == 0
                && merged[k].size as int % INIT_PAGE_SIZE as int == 0,
            forall|k: int| #![auto] 0 <= k < merged.len() as int ==>
                merged[k].spec_end() <= INIT_MEMORY_SIZE as int,
            forall|k: int| #![auto] 0 <= k < mmio_regions.len() as int ==>
                mmio_regions[k].spec_is_valid(),
            forall|k: int| #![auto] 0 <= k < mmio_regions.len() as int ==>
                mmio_regions[k].start as int % INIT_PAGE_SIZE as int == 0
                && mmio_regions[k].size as int % INIT_PAGE_SIZE as int == 0,
            forall|k: int| #![auto] 0 <= k < mmio_regions.len() as int ==>
                mmio_regions[k].spec_end() <= INIT_MEMORY_SIZE as int,
        decreases mmio_regions.len() - j,
    {
        merged.push(MemRegion { start: mmio_regions[j].start, size: mmio_regions[j].size, is_mmio: mmio_regions[j].is_mmio });
        j = j + 1;
    }
    merged
}
