pub fn store_with_ghost(
    index: usize,
    value: usize,
    Tracked(ghost): Tracked<&mut KernelRedZoneGhost>,
) -> (result: Result<(), Error>)
    requires
        old(ghost).inv(),
    ensures
        result.is_ok() ==> {
            &&& spec_is_valid_index(index as int)
            &&& ghost.view() == spec_store_effect(old(ghost).view(), index as int, value as int)
            &&& ghost.inv()
        },
        result.is_err() ==> {
            &&& !spec_is_valid_index(index as int)
            &&& ghost.view() == old(ghost).view()
        },
{
    let res = store(index, value);
    if res.is_ok() {
        proof {
            lemma_valid_index_in_bounds(ghost.view, index as int);
            lemma_update_preserves_well_formed(ghost.view, index as int, value as int);
            ghost.view = spec_store_effect(ghost.view, index as int, value as int);
        }
    }
    res
}
