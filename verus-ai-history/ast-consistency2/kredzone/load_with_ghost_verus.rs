pub fn load_with_ghost(
    index: usize,
    Tracked(ghost): Tracked<&KernelRedZoneGhost>,
) -> (result: Result<usize, Error>)
    requires
        ghost.inv(),
    ensures
        result.is_ok() ==> {
            &&& spec_is_valid_index(index as int)
            &&& result.unwrap() as int == spec_load_result(ghost.view(), index as int)
        },
        result.is_err() ==> !spec_is_valid_index(index as int),
{
    let res = load(index);
    proof {
        if res.is_ok() {
            // Trust boundary: see axiom_volatile_read_consistency for justification.
            axiom_volatile_read_consistency(ghost.view, res.unwrap(), index as int);
        }
    }
    res
}
