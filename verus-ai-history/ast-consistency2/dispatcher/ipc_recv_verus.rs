fn ipc_recv(tid: i64, pid: i64, arg0: u32) -> (result: SleepableOutcome)
    ensures result.wf(), result.succeeded ==> result.value == 0,
{ unimplemented!() }
