fn pm_giveup() -> (result: FallibleOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }
