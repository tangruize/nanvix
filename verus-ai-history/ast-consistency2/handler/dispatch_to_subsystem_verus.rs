pub fn dispatch_to_subsystem(kcall_number: u32) -> (result: HandlerKcallResult)
    ensures
        // Error results carry a non-zero error code.
        result.is_error ==> result.error_code != 0i32,
{
    unimplemented!()
}
