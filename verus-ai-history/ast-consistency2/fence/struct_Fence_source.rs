pub struct Fence {
    /// Number of signals received.
    count: AtomicUsize,
    /// Total number of signals to wait for.
    total: usize,
}
