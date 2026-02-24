pub struct IterationResult {
    /// The work state after the iteration.
    pub work_state: HandlerWorkState,
    /// Whether the CPU should yield.
    pub should_yield: bool,
    /// Whether the loop should terminate (INITD exited).
    pub should_terminate: bool,
    /// The exit status (meaningful only when `should_terminate` is true).
    pub exit_status: u32,
    /// The PID that triggered termination (meaningful only when `should_terminate` is true).
    pub initd_pid: u32,
    /// Whether a zombie was found in this iteration's harvest phase.
    pub harvest_found: bool,
    /// Whether the harvest phase encountered an error.
    pub harvest_error: bool,
    /// PID of the harvested zombie (meaningful when `harvest_found` is true).
    pub harvest_pid: u32,
    /// Whether the harvested zombie was INITD.
    pub harvest_is_initd: bool,
}
