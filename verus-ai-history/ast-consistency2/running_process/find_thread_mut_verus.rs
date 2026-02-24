    pub fn find_thread_mut(&mut self, tid: u64) -> (result: Ghost<Option<int>>)
        requires
            old(self).inv(),
        ensures
            result@ == old(self)@.find_thread(tid as int),
            // Frame: find_thread_mut does not change any modeled fields.
            self@ == old(self)@,
            self.inv() == old(self).inv(),
    {
        Ghost(old(self)@.find_thread(tid as int))
    }
