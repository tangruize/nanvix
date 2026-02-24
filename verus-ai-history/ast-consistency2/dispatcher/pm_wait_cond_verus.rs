fn pm_wait_cond(pid: i64, tid: i64, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> (result: SleepableOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }
