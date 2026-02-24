fn pm_get_tid() -> (result: FallibleOutcome)
    ensures result.wf(), result.succeeded ==> (result.value >= 0 && result.value <= i32::MAX as i64),
{ unimplemented!() }
