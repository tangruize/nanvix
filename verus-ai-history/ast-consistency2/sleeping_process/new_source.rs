    pub(super) fn new(
        process: Box<ProcessState>,
        sleeping_threads: NonEmptyVecDeque<SleepingThread>,
        zombie_threads: Option<NonEmptyVecDeque<ZombieThread>>,
    ) -> Self {
        Self {
            state: process,
            sleeping_threads,
            zombie_threads,
        }
    }
