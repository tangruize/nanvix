pub struct ThreadState {
    /// Thread identifier (verified dependency).
    pub id: ThreadIdentifier,
    /// Abstract kernel stack resource token (None = absent).
    /// Models `Option<KernelStack>` with identity preservation.
    pub kernel_stack: Option<int>,
    /// Abstract user stack resource token (None = absent).
    /// Models `Option<UserStack>` with identity preservation.
    pub user_stack: Option<int>,
    /// Optional base address for the user-space thread data area.
    pub user_tda: Option<int>,
    /// Interrupt reason tag, if any.
    pub interrupt_reason: Option<int>,
    /// Number of locked mutexes held by this thread.
    pub locked_mutex_count: usize,
    /// Concrete set of locked mutex addresses, faithfully modeling the
    /// original `BTreeMap<MutexAddress, MutexGuard>` per-key semantics.
    /// Backed by `Vec<u64>`; the `View` maps to `Set<int>` via `seq_to_set`.
    pub locked_mutex_set: Vec<u64>,
}
