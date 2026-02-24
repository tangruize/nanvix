pub fn should_yield(state: &HandlerWorkState) -> (result: bool)
    ensures
        result == (!state.kcall_handled && !state.message_received && !state.harvested_process),
{
    !state.kcall_handled && !state.message_received && !state.harvested_process
}
