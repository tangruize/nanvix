pub struct SleepingProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: u64,
    /// Sleeping thread IDs (non-empty).
    pub sleeping_thread_ids: Vec<u64>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Vec<u64>,
}
