pub fn signal_handled(result: &HandlerKcallResult)
    ensures
        // Signaling completes (may fail internally, but the call is
        // considered handled regardless). See handle_kcall_phase().
        true,
{
    unimplemented!()
}
