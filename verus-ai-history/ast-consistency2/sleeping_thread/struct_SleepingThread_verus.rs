pub struct SleepingThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// Optional alarm time for waking up the thread (abstract timestamp).
    pub alarm: Option<int>,
}
