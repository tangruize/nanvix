    pub fn state_mut(&mut self) -> &mut ProcessState {
        match self {
            ProcessRefMut::Runnable(process) => process.state_mut(),
            ProcessRefMut::Running(process) => process.state_mut(),
            ProcessRefMut::Sleeping(process) => process.state_mut(),
            ProcessRefMut::Interrupted(process) => process.state_mut(),
            ProcessRefMut::Zombie(process) => process.state_mut(),
        }
    }
