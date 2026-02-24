    pub(super) fn new(
        process: Box<ProcessState>,
        interrupted_threads: NonEmptyVecDeque<InterruptedThread>,
        zombie_threads: Option<NonEmptyVecDeque<ZombieThread>>,
    ) -> Self {
        Self {
            state: process,
            sleeping_threads: None,
            interrupted_threads,
            zombie_threads,
        }
    }
