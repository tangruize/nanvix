pub struct ZombieProcess {
    /// Process identifier.
    pub pid: u64,
    /// Zombie thread IDs (non-empty).
    pub zombie_thread_ids: Vec<u64>,
    /// Exit status.
    pub status: u64,
}
