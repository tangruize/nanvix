pub struct SleepingThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// The alarm time (abstract timestamp, None = no alarm).
    pub alarm: Option<int>,
}
