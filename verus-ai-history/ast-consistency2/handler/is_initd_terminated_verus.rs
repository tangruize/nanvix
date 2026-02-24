pub fn is_initd_terminated(harvest: &ZombieHarvestResult) -> (result: bool)
    ensures
        result == (harvest.found && harvest.is_initd),
{
    harvest.found && harvest.is_initd
}
