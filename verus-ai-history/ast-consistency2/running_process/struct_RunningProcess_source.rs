pub struct RunningProcess {
    /// Process state.
    state: Box<ProcessState>,
    /// Running thread.
    running: RunningThread,
    /// Ready threads.
    ready: Option<NonEmptyVecDeque<ReadyThread>>,
    /// Interrupted threads.
    interrupted_threads: Option<NonEmptyVecDeque<InterruptedThread>>,
    /// Sleeping threads.
    sleeping_threads: Option<NonEmptyVecDeque<SleepingThread>>,
    /// Zombie threads.
    zombie: Option<NonEmptyVecDeque<ZombieThread>>,
}
