pub struct Mutex {
    /// Lock state: `true` means locked, `false` means unlocked.
    pub locked: bool,
    /// Identity for distinguishing mutex instances.
    /// Callers must provide a unique `id` per instance at construction time.
    pub id: usize,
    /// Tracking of whether a `MutexToken` is currently outstanding.
    pub token_issued: bool,
}
