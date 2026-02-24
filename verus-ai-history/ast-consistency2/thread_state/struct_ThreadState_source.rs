pub struct ThreadState {
    /// Thread identifier.
    id: ThreadIdentifier,
    /// Kernel stack.
    kernel_stack: Option<KernelStack>,
    /// User stack.
    user_stack: Option<UserStack>,
    /// Condition variable for join.
    join_cond: Condvar,
    /// Execution context.
    context: Pin<Box<ContextInformation>>,
    /// Optional base address for the user-space thread data area.
    user_tda: Option<VirtualAddress>,
    /// Lookup table of locked mutexes.
    locked_mutexes: BTreeMap<MutexAddress, MutexGuard>,
    /// Interrupt reason, if any.
    interrupt_reason: Option<InterruptReason>,
    /// FPU state.
    fpu_state: Pin<Box<FpuState>>,
}
