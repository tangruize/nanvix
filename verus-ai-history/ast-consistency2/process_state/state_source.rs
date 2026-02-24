    pub fn state(&self) -> &ProcessState {
        match self {
            ProcessRef::Runnable(process) => process.state(),
            ProcessRef::Running(process) => process.state(),
            ProcessRef::Sleeping(process) => process.state(),
            ProcessRef::Interrupted(process) => process.state(),
            ProcessRef::Zombie(process) => process.state(),
        }
    }
