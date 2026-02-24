    pub fn new(pid: ProcessIdentifier, ready_thread: ReadyThread, vmem: Vmem) -> Self {
        Self {
            state: Box::new(ProcessState::new(pid, vmem)),
            ready_threads: NonEmptyVecDeque::new(ready_thread),
            interrupted_threads: None,
            sleeping_threads: None,
            zombie_threads: None,
        }
    }
