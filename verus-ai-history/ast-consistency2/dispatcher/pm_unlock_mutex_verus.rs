fn pm_unlock_mutex(pid: i64, tid: i64, arg0: u32) -> (result: FallibleOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }
