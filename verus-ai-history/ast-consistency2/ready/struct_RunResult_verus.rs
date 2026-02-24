pub struct RunResult {
    /// The running thread with interrupt reason cleared.
    pub running: RunningThread,
    /// The interrupt reason extracted from the state (if any).
    pub interrupt_reason: Option<int>,
    /// The user-space thread data area address (if any).
    pub user_tda: Option<int>,
}
