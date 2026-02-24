    pub fn state_mut(&mut self) -> (result: u64)
        ensures
            result as int == self@.pid,
            self@.pid == old(self)@.pid,
            self@.sleeping_thread_ids =~= old(self)@.sleeping_thread_ids,
            self@.zombie_thread_ids =~= old(self)@.zombie_thread_ids,
    {
        unimplemented!()
    }
