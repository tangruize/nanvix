pub fn is_user_region(
    valid: bool,
    Ghost(ghost_addr): Ghost<nat>,
    Ghost(ghost_size): Ghost<nat>,
) -> (result: bool)
    ensures
        result == valid,
{
    unimplemented!()
}
