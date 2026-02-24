fn diverge_after_exit() -> (result: DispatchResult)
    ensures
        false, // This function diverges (never returns).
{
    panic!("diverge_after_exit: killed path divergence")
}
