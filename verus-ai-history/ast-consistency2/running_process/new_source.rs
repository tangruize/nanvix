    pub(super) fn new(
        state: Box<ProcessState>,
        running: RunningThread,
        ready: Option<NonEmptyVecDeque<ReadyThread>>,
        interrupted: Option<NonEmptyVecDeque<InterruptedThread>>,
        sleeping: Option<NonEmptyVecDeque<SleepingThread>>,
        zombie: Option<NonEmptyVecDeque<ZombieThread>>,
    ) -> Self {
        Self {
            state,
            running,
            ready,
            interrupted_threads: interrupted,
            sleeping_threads: sleeping,
            zombie,
        }
    }
