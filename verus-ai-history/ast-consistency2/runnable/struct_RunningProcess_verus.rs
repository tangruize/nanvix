pub struct RunningProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: ProcessIdentifier,
    /// The running thread ID.
    pub running_thread_id: i64,
    /// Remaining ready thread IDs.
    pub ready_thread_ids: Vec<i64>,
    /// Interrupted thread IDs.
    pub interrupted_thread_ids: Vec<i64>,
    /// Sleeping thread IDs.
    pub sleeping_thread_ids: Vec<i64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Vec<i64>,
    /// Interrupt reason from the previous run (unconstrained).
    /// Models `Option<InterruptReason>` from the original return type.
    /// Downstream modules can constrain this value in their own specs.
    pub interrupt_reason: i64,
}
