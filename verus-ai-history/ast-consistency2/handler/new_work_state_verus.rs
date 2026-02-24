pub fn new_work_state() -> (result: HandlerWorkState)
    ensures
        !result.kcall_handled,
        !result.message_received,
        !result.harvested_process,
{
    HandlerWorkState {
        kcall_handled: false,
        message_received: false,
        harvested_process: false,
    }
}
