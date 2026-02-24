    pub(super) fn from_state(
        state: Box<ProcessState>,
        ready_threads: NonEmptyVecDeque<ReadyThread>,
        interrupted_threads: Option<NonEmptyVecDeque<InterruptedThread>>,
        sleeping_threads: Option<NonEmptyVecDeque<SleepingThread>>,
        zombie_threads: Option<NonEmptyVecDeque<ZombieThread>>,
    ) -> Self {
        Self {
            state,
            ready_threads,
            interrupted_threads,
            sleeping_threads,
            zombie_threads,
        }
    }
