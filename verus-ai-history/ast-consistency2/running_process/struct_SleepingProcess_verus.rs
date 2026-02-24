pub struct SleepingProcess {
    /// Process identifier.
    pub pid: u64,
    /// Sleeping thread IDs (non-empty).
    pub sleeping_thread_ids: Vec<u64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Vec<u64>,
}
