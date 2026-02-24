struct ProcessManagerInner {
    /// Is this platform interrupt capable?
    interrupt_capable: bool,
    /// Reason for the last interrupt.
    interrupt_reason: Option<InterruptReason>,
    /// Next process identifier.
    next_pid: ProcessIdentifier,
    /// Running process.
    running: Option<RunningProcess>,
    /// Ready processes.
    ready: LinkedList<RunnableProcess>,
    /// Suspended processes.
    suspended: LinkedList<SleepingProcess>,
    /// Interrupted processes.
    interrupted: LinkedList<InterruptedProcess>,
    /// Zombie processes.
    zombies: LinkedList<ZombieProcess>,
    /// Thread manager.
    tm: ThreadManager,
    /// Number of messages buffered (not yet consumed).
    number_buffered_messages: usize,
}
