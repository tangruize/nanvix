pub struct LifecycleStepResult {
    /// Whether the handler loop terminated (INITD exited).
    pub terminated: bool,
    /// The exit status (meaningful only when `terminated` is true).
    /// This value originates from `harvest_zombies()` (T2) and is
    /// unconstrained — its correctness depends on ProcessManager state.
    pub exit_status: u32,
    /// The PID that triggered termination (meaningful only when `terminated`).
    /// Proved to equal INITD (1) when terminated.
    pub termination_pid: u32,
}
