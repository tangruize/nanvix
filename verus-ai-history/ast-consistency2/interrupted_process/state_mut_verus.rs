    pub fn state_mut(&mut self) -> (result: u64)
        requires
            old(self).wf(),
        ensures
            result as int == self@.pid,
            self@.pid == old(self)@.pid,
            self@.interrupted_thread_ids =~= old(self)@.interrupted_thread_ids,
            self@.sleeping_thread_ids =~= old(self)@.sleeping_thread_ids,
            self@.zombie_thread_ids =~= old(self)@.zombie_thread_ids,
            self.wf(),
    {
        proof {
            reveal(InterruptedProcess::wf);
        }
        self.pid
    }
