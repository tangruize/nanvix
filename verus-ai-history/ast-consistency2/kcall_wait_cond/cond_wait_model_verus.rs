pub fn cond_wait_model(
    has_alarm: bool,
    timeout_s: Ghost<u32>,
    timeout_ns: Ghost<u32>,
) -> (result: CondWaitOutcomeModel)
    requires
        has_alarm ==> spec_is_finite_timeout(timeout_s@ as nat, timeout_ns@ as nat),
    ensures
        // TimedOut can only occur when an alarm is set.
        result matches CondWaitOutcomeModel::TimedOut ==> has_alarm,
{
    unimplemented!()
}
