pub struct RunnableProcess {
    /// Process identifier.
    pub pid: u64,
    /// Ready thread IDs (non-empty).
    pub ready_thread_ids: Vec<u64>,
    /// Interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Vec<u64>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Vec<u64>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Vec<u64>,
}
