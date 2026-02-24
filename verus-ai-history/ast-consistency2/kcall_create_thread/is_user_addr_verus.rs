pub fn is_user_addr(
    valid: bool,
    Ghost(ghost_addr): Ghost<nat>,
) -> (result: bool)
    ensures
        result == valid,
{
    unimplemented!()
}
