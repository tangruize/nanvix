    pub fn harvest(self) -> (result: (Option<int>, Option<int>))
        requires
            self.wf(),
        ensures
            result.0 == self@.spec_kernel_stack(),
            result.1 == self@.spec_user_stack(),
    {
        proof { reveal(ZombieThread::wf); }
        let mut state: ThreadState = self.state;
        let kstack: Option<int> = state.take_kernel_stack();
        let ustack: Option<int> = state.take_user_stack();
        (kstack, ustack)
    }
