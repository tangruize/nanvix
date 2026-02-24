pub struct HandlerKcallPhaseResult {
    /// Whether a kcall was handled.
    pub kcall_handled: bool,
    /// Whether the handled kcall was an InvalidSysCall.
    pub was_invalid_syscall: bool,
}
