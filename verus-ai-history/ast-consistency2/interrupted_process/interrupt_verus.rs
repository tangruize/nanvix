pub fn interrupt(sleeping_tid: u64) -> (result: (u64, u64))
    ensures
        result.0 == sleeping_tid,
        result.1 as int == InterruptedProcess::INTERRUPT_REASON_KILLED(),
{
    (sleeping_tid, 0u64)
}
