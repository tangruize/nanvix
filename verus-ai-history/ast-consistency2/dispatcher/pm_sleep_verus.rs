fn pm_sleep(arg0: u32, arg1: u32) -> (result: SleepableOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }
