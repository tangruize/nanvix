    pub fn bury(self) -> (NonEmptyVecDeque<ZombieThread>, Box<ProcessState>, ExitStatus) {
        (self.zombie_threads, self.process, self.status)
    }
