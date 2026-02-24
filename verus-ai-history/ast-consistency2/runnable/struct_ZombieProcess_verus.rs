pub struct ZombieProcess {
    /// Process identifier.
    pub pid: ProcessIdentifier,
    /// Zombie thread IDs (non-empty).
    pub zombie_thread_ids: Vec<i64>,
    /// Exit status.
    pub status: i64,
}
