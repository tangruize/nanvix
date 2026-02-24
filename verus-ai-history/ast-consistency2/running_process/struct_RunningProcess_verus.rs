pub struct RunningProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: u64,
    /// Running thread ID.
    pub running_thread_id: u64,
    /// Concrete vector of ready thread IDs.
    pub ready_thread_ids: Vec<u64>,
    /// Concrete vector of interrupted thread IDs.
    pub interrupted_thread_ids: Vec<u64>,
    /// Concrete vector of sleeping thread IDs.
    pub sleeping_thread_ids: Vec<u64>,
    /// Concrete vector of zombie thread IDs.
    pub zombie_thread_ids: Vec<u64>,
    /// Exec-level count of ready threads.
    pub ready_count: u64,
    /// Exec-level count of interrupted threads.
    pub interrupted_count: u64,
    /// Exec-level count of sleeping threads.
    pub sleeping_count: u64,
    /// Exec-level count of zombie threads.
    pub zombie_count: u64,
}
