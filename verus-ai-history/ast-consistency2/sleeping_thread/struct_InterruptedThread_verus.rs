pub struct InterruptedThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// The interrupt reason tag (0 = Killed, 1 = TimedOut).
    pub reason: int,
}
