pub fn harvest_zombies() -> (result: ZombieHarvestResult)
    ensures
        result.is_initd ==> (result.found && result.pid == 1u32),
        (result.found && result.pid == 1u32) ==> result.is_initd,
        result.error ==> !result.found,
{
    unimplemented!()
}
