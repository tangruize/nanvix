pub fn sort_regions_by_start(regions: &mut Vec<MemRegion>)
    requires
        forall|i: int| #![auto] 0 <= i < old(regions).len() as int ==>
            old(regions)[i].spec_is_valid(),
        forall|i: int| #![auto] 0 <= i < old(regions).len() as int ==>
            old(regions)[i].start as int % INIT_PAGE_SIZE as int == 0
            && old(regions)[i].size as int % INIT_PAGE_SIZE as int == 0,
        forall|i: int| #![auto] 0 <= i < old(regions).len() as int ==>
            old(regions)[i].spec_end() <= INIT_MEMORY_SIZE as int,
    ensures
        regions.len() == old(regions).len(),
        // Permutation: output is a rearrangement of input elements.
        regions@.to_multiset() =~= old(regions)@.to_multiset(),
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].spec_is_valid(),
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].start as int % INIT_PAGE_SIZE as int == 0
            && regions[i].size as int % INIT_PAGE_SIZE as int == 0,
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].spec_end() <= INIT_MEMORY_SIZE as int,
        forall|i: int, j: int|
            #![trigger regions[i], regions[j]]
            0 <= i < j < regions.len() as int ==>
            regions[i].spec_start() <= regions[j].spec_start(),
{
    unimplemented!()
}
