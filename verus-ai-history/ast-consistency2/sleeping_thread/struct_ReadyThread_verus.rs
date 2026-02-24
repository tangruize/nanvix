pub struct ReadyThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// Admission time (abstract timestamp >= 0, set by clock_now() in real impl).
    /// Included to match the real ReadyThread structure for cross-module compatibility.
    pub admission_time: int,
}
