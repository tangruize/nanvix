pub struct Condvar {
    /// Concrete queue length.
    pub len: usize,
    /// Concrete FIFO queue of (pid_value, tid_value) pairs.
    pub sleeping: Vec<(i32, i32)>,
}
