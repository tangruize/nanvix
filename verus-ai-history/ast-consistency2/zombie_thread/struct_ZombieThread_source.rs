pub struct ZombieThread {
    /// Exit status of the terminated thread.
    status: ExitStatus,
    /// Thread state.
    state: Box<ThreadState>,
}
