    pub fn thread_state(&self) -> (result: &ThreadState)
        ensures
            result.spec_id() == self.spec_id(),
            result@ == self.spec_state()@,
    {
        match self {
            ThreadRefMutModel::Ready(state) => state,
            ThreadRefMutModel::Running(state) => state,
            ThreadRefMutModel::Sleeping(state) => state,
            ThreadRefMutModel::Interrupted(state) => state,
            ThreadRefMutModel::Zombie(state) => state,
        }
    }
