pub fn init(regions: &Vec<MemRegion>) -> (result: (Vec<usize>, Ghost<Seq<PageMapping>>))
    requires
        // All regions are valid.
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].spec_is_valid(),
        // Regions are sorted by start address.
        forall|i: int, j: int|
            #![trigger regions[i], regions[j]]
            0 <= i < j < regions.len() as int ==>
            regions[i].spec_start() <= regions[j].spec_start(),
        // Regions are non-overlapping.
        forall|i: int, j: int|
            #![trigger regions[i], regions[j]]
            0 <= i < j < regions.len() as int ==>
            regions[i].spec_end() <= regions[j].spec_start(),
        // Regions have page-aligned start and size.
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].start as int % INIT_PAGE_SIZE as int == 0
            && regions[i].size as int % INIT_PAGE_SIZE as int == 0,
        // All regions fit within kernel memory (models is_last_kernel_page break).
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].spec_end() <= INIT_MEMORY_SIZE as int,
    ensures
        // All output bases are aligned.
        forall|i: int| #![auto] 0 <= i < result.0.len() as int ==>
            result.0[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
        // Output bases are strictly increasing (ordered + unique).
        forall|i: int, j: int|
            #![trigger result.0[i], result.0[j]]
            0 <= i < j < result.0.len() as int ==>
            (result.0[i] as int) < (result.0[j] as int),
        // Number of page table bases <= total pages across all regions.
        result.0.len() as int <= spec_total_pages(regions@, regions.len() as int),
        // Mapping coverage: exactly spec_total_pages entries produced.
        result.1@.len() == spec_total_pages(regions@, regions.len() as int),
        // Mapping correctness: paddr matches spec_init_paddr for every page.
        forall|k: int| #![auto] 0 <= k < result.1@.len() ==>
            result.1@[k].paddr == spec_init_paddr(
                result.1@[k].vaddr, result.1@[k].region_start, result.1@[k].is_mmio),
        // Identity mapping: non-MMIO pages have paddr == vaddr.
        forall|k: int| #![auto] 0 <= k < result.1@.len() ==>
            (!result.1@[k].is_mmio ==> result.1@[k].paddr == result.1@[k].vaddr),
        // Alignment: all mapped vaddrs are page-aligned.
        forall|k: int| #![auto] 0 <= k < result.1@.len() ==>
            result.1@[k].vaddr % INIT_PAGE_SIZE as int == 0,
        // Alignment: all mapped paddrs are page-aligned.
        forall|k: int| #![auto] 0 <= k < result.1@.len() ==>
            result.1@[k].paddr % INIT_PAGE_SIZE as int == 0,
        // No double-mapping: all mapped vaddrs are strictly increasing.
        forall|i: int, j: int|
            #![trigger result.1@[i], result.1@[j]]
            0 <= i < j < result.1@.len() ==>
            result.1@[i].vaddr < result.1@[j].vaddr,
        // Permissions: all mappings have init-time permission attributes.
        forall|k: int| #![auto] 0 <= k < result.1@.len() ==>
            spec_has_init_permissions(result.1@[k]),
{
    let mut all_bases: Vec<usize> = Vec::new();
    let mut last_base: Option<usize> = None;
    let mut r_idx: usize = 0;
    let ghost mut total_mapped: int = 0;
    let ghost mut mappings: Seq<PageMapping> = Seq::empty();
    let ghost mut last_mapped_vaddr: Option<int> = None;

    proof {
        VirtProofs::lemma_total_pages_nonneg(regions@, 0);
    }

    while r_idx < regions.len()
        invariant
            0 <= r_idx <= regions.len(),
            // Forward regions info.
            forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
                regions[i].spec_is_valid(),
            forall|i: int, j: int|
                #![trigger regions[i], regions[j]]
                0 <= i < j < regions.len() as int ==>
                regions[i].spec_start() <= regions[j].spec_start(),
            forall|i: int, j: int|
                #![trigger regions[i], regions[j]]
                0 <= i < j < regions.len() as int ==>
                regions[i].spec_end() <= regions[j].spec_start(),
            forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
                regions[i].start as int % INIT_PAGE_SIZE as int == 0
                && regions[i].size as int % INIT_PAGE_SIZE as int == 0,
            forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
                regions[i].spec_end() <= INIT_MEMORY_SIZE as int,
            // All accumulated bases are aligned.
            forall|i: int| #![auto] 0 <= i < all_bases.len() as int ==>
                all_bases[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
            // Accumulated bases are strictly increasing.
            forall|i: int, j: int|
                #![trigger all_bases[i], all_bases[j]]
                0 <= i < j < all_bases.len() as int ==>
                (all_bases[i] as int) < (all_bases[j] as int),
            // last_base is aligned when present.
            last_base.is_some() ==>
                last_base.unwrap() as int % INIT_PGTAB_ALIGNMENT as int == 0,
            // last_base >= all accumulated bases (enables strictly-increasing pushes).
            last_base.is_some() ==> forall|i: int| #![auto]
                0 <= i < all_bases.len() as int ==>
                all_bases[i] as int <= last_base.unwrap() as int,
            // When last_base is None, all_bases is empty.
            last_base.is_none() ==> all_bases.len() == 0,
            // Cross-region: last_base <= pgtab_base of the next region to process.
            last_base.is_some() && r_idx < regions.len() as int ==>
                last_base.unwrap() as int
                    <= spec_pgtab_base(regions[r_idx as int].spec_start()),
            // Ghost counter: total_mapped tracks pages processed so far.
            total_mapped == spec_total_pages(regions@, r_idx as int),
            // Number of bases <= total pages mapped.
            all_bases.len() as int <= total_mapped,
            // Ghost mapping tracker: one entry per page processed.
            mappings.len() == total_mapped,
            // Ghost mapping: paddr matches spec_init_paddr for every page.
            forall|k: int| #![auto] 0 <= k < mappings.len() ==>
                mappings[k].paddr == spec_init_paddr(
                    mappings[k].vaddr, mappings[k].region_start, mappings[k].is_mmio),
            // Ghost mapping: non-MMIO pages are identity-mapped.
            forall|k: int| #![auto] 0 <= k < mappings.len() ==>
                (!mappings[k].is_mmio ==> mappings[k].paddr == mappings[k].vaddr),
            // Ghost mapping: all vaddrs are page-aligned.
            forall|k: int| #![auto] 0 <= k < mappings.len() ==>
                mappings[k].vaddr % INIT_PAGE_SIZE as int == 0,
            // Ghost mapping: all paddrs are page-aligned.
            forall|k: int| #![auto] 0 <= k < mappings.len() ==>
                mappings[k].paddr % INIT_PAGE_SIZE as int == 0,
            // Ghost mapping: vaddrs are strictly increasing (no double-mapping).
            forall|i: int, j: int|
                #![trigger mappings[i], mappings[j]]
                0 <= i < j < mappings.len() ==>
                mappings[i].vaddr < mappings[j].vaddr,
            // Ghost: last_mapped_vaddr tracks the most recent vaddr.
            mappings.len() > 0 ==> last_mapped_vaddr.is_some(),
            mappings.len() == 0 ==> last_mapped_vaddr.is_none(),
            last_mapped_vaddr.is_some() ==> forall|k: int| #![auto]
                0 <= k < mappings.len() ==>
                mappings[k].vaddr <= last_mapped_vaddr.unwrap(),
            // Ghost: cross-region vaddr ordering.
            last_mapped_vaddr.is_some() && r_idx < regions.len() as int ==>
                last_mapped_vaddr.unwrap() < regions[r_idx as int].spec_start(),
            // Ghost mapping: all mappings have init-time permissions.
            forall|k: int| #![auto] 0 <= k < mappings.len() ==>
                spec_has_init_permissions(mappings[k]),
        decreases regions.len() - r_idx,
    {
        let region: &MemRegion = &regions[r_idx];
        let page_count: usize = compute_region_page_count(region.size);
        let mut p_idx: usize = 0;

        while p_idx < page_count
            invariant
                0 <= p_idx <= page_count,
                page_count as int == region.size as int / INIT_PAGE_SIZE as int,
                page_count > 0,
                region.spec_is_valid(),
                region.start as int % INIT_PAGE_SIZE as int == 0,
                region.size as int % INIT_PAGE_SIZE as int == 0,
                region.spec_end() <= INIT_MEMORY_SIZE as int,
                // All accumulated bases are aligned.
                forall|i: int| #![auto] 0 <= i < all_bases.len() as int ==>
                    all_bases[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
                // Accumulated bases are strictly increasing.
                forall|i: int, j: int|
                    #![trigger all_bases[i], all_bases[j]]
                    0 <= i < j < all_bases.len() as int ==>
                    (all_bases[i] as int) < (all_bases[j] as int),
                // last_base is aligned when present.
                last_base.is_some() ==>
                    last_base.unwrap() as int % INIT_PGTAB_ALIGNMENT as int == 0,
                // last_base >= all accumulated bases.
                last_base.is_some() ==> forall|i: int| #![auto]
                    0 <= i < all_bases.len() as int ==>
                    all_bases[i] as int <= last_base.unwrap() as int,
                // When last_base is None, all_bases is empty.
                last_base.is_none() ==> all_bases.len() == 0,
                // After first page, last_base is Some.
                p_idx > 0 ==> last_base.is_some(),
                // last_base tracks the pgtab base of the previous page.
                p_idx > 0 ==> last_base.unwrap() as int == spec_pgtab_base(
                    spec_nth_page_addr(region.start as int, (p_idx - 1) as int)),
                // For p_idx == 0: if last_base is Some, it's <= this region's first pgtab base.
                p_idx == 0 && last_base.is_some() ==>
                    last_base.unwrap() as int
                        <= spec_pgtab_base(region.spec_start()),
                // Ghost counter: tracks pages in current region.
                total_mapped == spec_total_pages(regions@, r_idx as int) + p_idx as int,
                // Number of bases <= total pages mapped.
                all_bases.len() as int <= total_mapped,
                // Ghost mapping tracker: one entry per page processed.
                mappings.len() == total_mapped,
                // Ghost mapping: paddr matches spec_init_paddr for every page.
                forall|k: int| #![auto] 0 <= k < mappings.len() ==>
                    mappings[k].paddr == spec_init_paddr(
                        mappings[k].vaddr, mappings[k].region_start, mappings[k].is_mmio),
                // Ghost mapping: non-MMIO pages are identity-mapped.
                forall|k: int| #![auto] 0 <= k < mappings.len() ==>
                    (!mappings[k].is_mmio ==> mappings[k].paddr == mappings[k].vaddr),
                // Ghost mapping: all vaddrs are page-aligned.
                forall|k: int| #![auto] 0 <= k < mappings.len() ==>
                    mappings[k].vaddr % INIT_PAGE_SIZE as int == 0,
                // Ghost mapping: all paddrs are page-aligned.
                forall|k: int| #![auto] 0 <= k < mappings.len() ==>
                    mappings[k].paddr % INIT_PAGE_SIZE as int == 0,
                // Ghost mapping: vaddrs are strictly increasing (no double-mapping).
                forall|i: int, j: int|
                    #![trigger mappings[i], mappings[j]]
                    0 <= i < j < mappings.len() ==>
                    mappings[i].vaddr < mappings[j].vaddr,
                // Ghost: last_mapped_vaddr tracks the most recent vaddr.
                mappings.len() > 0 ==> last_mapped_vaddr.is_some(),
                mappings.len() == 0 ==> last_mapped_vaddr.is_none(),
                last_mapped_vaddr.is_some() ==> forall|k: int| #![auto]
                    0 <= k < mappings.len() ==>
                    mappings[k].vaddr <= last_mapped_vaddr.unwrap(),
                // Ghost: last_mapped_vaddr tracks the page vaddr.
                p_idx > 0 ==> last_mapped_vaddr == Some(
                    spec_nth_page_addr(region.start as int, (p_idx - 1) as int)),
                // Ghost: for p_idx == 0, last_mapped_vaddr < region start.
                p_idx == 0 && last_mapped_vaddr.is_some() ==>
                    last_mapped_vaddr.unwrap() < region.spec_start(),
                // Ghost mapping: all mappings have init-time permissions.
                forall|k: int| #![auto] 0 <= k < mappings.len() ==>
                    spec_has_init_permissions(mappings[k]),
            decreases page_count - p_idx,
        {
            proof {
                // Prove overflow safety for get_nth_page_addr.
                VirtProofs::lemma_page_iteration_covers_region(
                    region.start as int, region.size as int, p_idx as int);
            }
            let vaddr: usize = get_nth_page_addr(region.start, p_idx);
            let curr_base: usize = compute_pgtab_base(vaddr);

            // Compute the physical address for this page (functional completeness).
            // Non-MMIO: paddr == vaddr (identity mapping).
            // MMIO: paddr == spec_mmio_paddr(region.start) (constant across pages).
            let paddr: usize = get_page_paddr(vaddr, region.start, region.is_mmio);

            // Execute the page table map operation (side effect).
            // This models the original's page_table.map(vaddr, paddr) call.
            page_table_map_page(vaddr, paddr);

            proof {
                // Prove monotonicity: curr_base >= last_base (when present).
                if p_idx > 0 {
                    VirtProofs::lemma_consecutive_pages_ordered_bases(
                        region.start as int, (p_idx - 1) as int, p_idx as int);
                }
            }

            let should_add: bool = match last_base {
                None => true,
                Some(prev) => curr_base > prev,
            };

            if should_add {
                all_bases.push(curr_base);
            }
            last_base = Some(curr_base);

            proof {
                // Prove new vaddr > last_mapped_vaddr for strictly-increasing invariant.
                if last_mapped_vaddr.is_some() {
                    if p_idx > 0 {
                        // Within region: vaddr = start + p_idx*PS > start + (p_idx-1)*PS.
                        let ps: int = INIT_PAGE_SIZE as int;
                        let prev_vaddr: int = spec_nth_page_addr(region.start as int, (p_idx - 1) as int);
                        vstd::arithmetic::mul::lemma_mul_inequality(p_idx as int - 1, p_idx as int, ps);
                    }
                    // Cross-region (p_idx == 0): last_mapped_vaddr < region.start = vaddr.
                }
                last_mapped_vaddr = Some(vaddr as int);
                mappings = mappings.push(PageMapping {
                    vaddr: vaddr as int,
                    paddr: paddr as int,
                    region_start: region.start as int,
                    is_mmio: region.is_mmio,
                    present: true,
                    writable: true,
                    user_accessible: false,
                });
                total_mapped = total_mapped + 1;
            }

            p_idx = p_idx + 1;
        }

        proof {
            // Unfold spec_total_pages for the transition r_idx -> r_idx + 1.
            VirtProofs::lemma_total_pages_step(regions@, r_idx as int);
        }

        r_idx = r_idx + 1;

        proof {
            // Establish cross-region invariant for the next iteration.
            if r_idx < regions.len() {
                let last_page_idx: int = region.size as int / INIT_PAGE_SIZE as int - 1;
                let last_page: int = spec_nth_page_addr(region.start as int, last_page_idx);
                VirtProofs::lemma_page_iteration_covers_region(
                    region.start as int, region.size as int, last_page_idx);
                let next_start: int = regions[r_idx as int].spec_start();
                assert(last_page < region.spec_end());
                assert(region.spec_end() <= next_start);
                VirtProofs::lemma_sorted_addrs_sorted_pgtab_bases(last_page, next_start);
                // Establish last_mapped_vaddr < next region start.
                assert(last_mapped_vaddr == Some(last_page));
                assert(last_page < next_start);
            }
        }
    }

    (all_bases, Ghost(mappings))
}
