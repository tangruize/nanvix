pub struct InterruptedProcess {
    /// Process identifier.
    pub pid: u64,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Vec<u64>,
    /// Sleeping thread IDs (carried through resume).
    pub sleeping_thread_ids: Vec<u64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Vec<u64>,
}
