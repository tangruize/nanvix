pub struct MutexGuard {
    /// Reference to underlying mutex data.
    mutex: Arc<MutexInner>,
}
