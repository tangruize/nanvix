pub struct ReadyThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// Time when the thread was admitted to the ready queue.
    pub admission_time: int,
}
