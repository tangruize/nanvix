pub fn poll_scoreboard_full() -> (result: ScoreBoardPollResult)
    ensures
        // When no call is pending, the kcall_number has no meaning.
        // Default is 0 (Debug); the `has_call` gate in handle_kcall_phase
        // prevents this from being used.
        !result.has_call ==> result.kcall_number == 0u32,
        // Scoreboard errors never occur in practice (original: unreachable!()).
        // This matches the fail-stop semantics: if these paths execute,
        // the kernel panics. The verification assumes they don't.
        !result.has_error,
{
    unimplemented!()
}
