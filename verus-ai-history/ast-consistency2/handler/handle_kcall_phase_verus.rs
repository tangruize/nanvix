pub fn handle_kcall_phase(poll: &ScoreBoardPollResult) -> (result: HandlerKcallPhaseResult)
    ensures
        result.kcall_handled == poll.has_call,
        poll.has_call && spec_returns_invalid_syscall(poll.kcall_number) ==>
            result.was_invalid_syscall,
        poll.has_call && !spec_returns_invalid_syscall(poll.kcall_number) ==>
            !result.was_invalid_syscall,
{
    if poll.has_call {
        let is_invalid: bool = classify_and_check_invalid(poll.kcall_number);
        let kcall_result: HandlerKcallResult = if is_invalid {
            make_invalid_syscall_error()
        } else {
            dispatch_to_subsystem(poll.kcall_number)
        };
        signal_handled(&kcall_result);
        HandlerKcallPhaseResult {
            kcall_handled: true,
            was_invalid_syscall: is_invalid,
        }
    } else {
        HandlerKcallPhaseResult {
            kcall_handled: false,
            was_invalid_syscall: false,
        }
    }
}
