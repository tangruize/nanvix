pub struct Semaphore {
    /// Current count of available resources.
    value: AtomicUsize,
    /// Condition variable for threads waiting on the semaphore.
    sleeping: Condvar,
}
