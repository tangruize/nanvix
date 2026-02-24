pub struct InterruptedProcess {
    /// Process identifier.
    pub pid: ProcessIdentifier,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Vec<i64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Vec<i64>,
}
