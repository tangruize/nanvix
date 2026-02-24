pub struct LoopResult {
    /// Whether the handler loop terminated (INITD exited).
    pub terminated: bool,
    /// The exit status (meaningful only when `terminated` is true).
    /// Originates from `harvest_zombies()` (T2); its correctness depends
    /// on ProcessManager state and is outside verification scope.
    pub exit_status: u32,
    /// The PID that triggered termination (meaningful only when `terminated`).
    pub termination_pid: u32,
}
