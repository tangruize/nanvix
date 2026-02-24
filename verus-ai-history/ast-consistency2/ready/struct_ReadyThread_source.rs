pub struct ReadyThread {
    /// Thread state.
    state: Box<ThreadState>,
    /// Time when the thread was admitted to the ready queue.
    admission_time: SystemTime,
}
