pub struct SleepingThread {
    /// Thread state.
    state: Box<ThreadState>,
    /// Optional alarm time for waking up the thread.
    alarm: Option<SystemTime>,
}
