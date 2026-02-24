    pub(super) fn from_sleeping(
        process: Box<ProcessState>,
        sleeping_threads: Option<NonEmptyVecDeque<SleepingThread>>,
        interrupted_threads: NonEmptyVecDeque<InterruptedThread>,
        zombie_threads: Option<NonEmptyVecDeque<ZombieThread>>,
    ) -> Self {
        Self {
            state: process,
            sleeping_threads,
            interrupted_threads,
            zombie_threads,
        }
    }
