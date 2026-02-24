pub fn validate_regions(regions: &Vec<MemRegion>) -> (result: bool)
    requires
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].spec_is_valid(),
    ensures
        // Soundness: if validation passes, the sorted/non-overlapping properties hold.
        result ==> forall|i: int, j: int|
            #![trigger regions[i], regions[j]]
            0 <= i < j < regions.len() as int ==>
            regions[i].spec_start() <= regions[j].spec_start(),
        result ==> forall|i: int, j: int|
            #![trigger regions[i], regions[j]]
            0 <= i < j < regions.len() as int ==>
            regions[i].spec_end() <= regions[j].spec_start(),
        // Completeness: if regions are sorted and non-overlapping, validation passes.
        (forall|i: int| #![auto] 0 <= i < regions.len() as int - 1 ==>
            regions[i].spec_end() <= regions[i + 1].spec_start())
            ==> result,
{
    if regions.len() <= 1 {
        return true;
    }
    let mut i: usize = 0;
    while i < regions.len() - 1
        invariant
            0 <= i <= regions.len() - 1,
            regions.len() > 1,
            forall|k: int| #![auto] 0 <= k < regions.len() as int ==>
                regions[k].spec_is_valid(),
            // All checked pairs so far satisfy the ordering.
            forall|k: int| #![auto] 0 <= k < i as int ==>
                regions[k].spec_end() <= regions[k + 1].spec_start(),
            // Transitivity: checked pairs imply full sorted + non-overlapping.
            forall|a: int, b: int|
                #![trigger regions[a], regions[b]]
                0 <= a < b <= i as int ==>
                regions[a].spec_end() <= regions[b].spec_start(),
            forall|a: int, b: int|
                #![trigger regions[a], regions[b]]
                0 <= a < b <= i as int ==>
                regions[a].spec_start() <= regions[b].spec_start(),
        decreases regions.len() - 1 - i,
    {
        // Check: region[i].end > region[i+1].start, i.e. overlap.
        // Use subtraction to avoid overflow: start + size could overflow usize,
        // but spec_is_valid ensures start + size - 1 <= usize::MAX.
        // Compare as: regions[i+1].start - regions[i].start < regions[i].size
        // (safe because regions with valid starts won't underflow).
        if regions[i + 1].start < regions[i].start
           || regions[i + 1].start - regions[i].start < regions[i].size {
            // Overlap detected: regions[i].end > regions[i+1].start.
            proof {
                assert(regions[i as int].spec_end() > regions[i as int + 1].spec_start());
            }
            return false;
        }
        proof {
            // Establish transitivity for new index.
            assert(regions[i as int].spec_end() <= regions[i as int + 1].spec_start());
            assert forall|a: int, b: int|
                #![trigger regions[a], regions[b]]
                0 <= a < b <= i as int + 1
            implies
                regions[a].spec_end() <= regions[b].spec_start()
            by {
                if b == i as int + 1 {
                    if a < i as int {
                        assert(regions[a].spec_end() <= regions[a + 1].spec_start());
                        assert(regions[a + 1].spec_start() <= regions[a + 1].spec_end());
                    }
                }
            }
            assert forall|a: int, b: int|
                #![trigger regions[a], regions[b]]
                0 <= a < b <= i as int + 1
            implies
                regions[a].spec_start() <= regions[b].spec_start()
            by {
                if b == i as int + 1 {
                    if a < i as int {
                        assert(regions[a].spec_end() <= regions[a + 1].spec_start());
                    }
                }
            }
        }
        i = i + 1;
    }
    true
}
