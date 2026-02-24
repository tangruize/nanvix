    pub(super) fn new(
        process: Box<ProcessState>,
        zombie_threads: NonEmptyVecDeque<ZombieThread>,
        status: ExitStatus,
    ) -> Self {
        Self {
            zombie_threads,
            process,
            status,
        }
    }
