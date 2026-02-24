pub struct Fence {
    /// Number of signals received so far.
    pub count: usize,
    /// Total number of signals to wait for.
    pub total: usize,
}
