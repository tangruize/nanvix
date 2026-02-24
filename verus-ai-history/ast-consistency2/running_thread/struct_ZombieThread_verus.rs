pub struct ZombieThread {
    /// The underlying thread state.
    pub state: ThreadState,
    /// The exit status (abstract int).
    pub status: int,
}
