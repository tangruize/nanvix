pub struct ZombieThread {
    /// The exit status of the terminated thread (abstract int tag).
    pub status: int,
    /// The underlying thread state.
    pub state: ThreadState,
}
