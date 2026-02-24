pub struct MutexInner {
    /// Locked?
    locked: AtomicBool,
    /// Threads that are sleeping on the mutex.
    sleeping: Condvar,
}
