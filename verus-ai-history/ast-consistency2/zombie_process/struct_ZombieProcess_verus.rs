pub struct ZombieProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: u64,
    /// Concrete sequence of zombie thread IDs (non-empty).
    pub zombie_thread_ids: Vec<u64>,
    /// Exit status.
    pub status: i64,
    /// Exec-level count of zombie threads.
    pub zombie_count: u64,
}
