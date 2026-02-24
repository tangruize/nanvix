fn pm_signal_cond(pid: i64, tid: i64, arg0: u32, broadcast: bool) -> (result: FallibleOutcome)
    ensures result.wf(), result.succeeded ==> result.value >= 0,
{ unimplemented!() }
