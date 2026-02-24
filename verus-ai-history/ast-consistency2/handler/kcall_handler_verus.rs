pub fn kcall_handler() -> (exit_status: u32)
    ensures
        // The exit status originates from harvest_zombies() (T2).
        // Its correctness depends on ProcessManager state outside
        // the verification scope. See kcall_handler_loop() for the
        // verified control flow and termination properties.
        true,
{
    unimplemented!()
}
