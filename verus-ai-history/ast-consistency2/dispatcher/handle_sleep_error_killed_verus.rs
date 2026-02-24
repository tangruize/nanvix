pub fn handle_sleep_error_killed() -> (result: DispatchResult)
    ensures
        false, // This function diverges (never returns).
{
    pm_exit_interrupted();
    diverge_after_exit()
}
