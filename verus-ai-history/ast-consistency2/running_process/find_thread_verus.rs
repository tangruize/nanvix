    pub fn find_thread(&self, tid: u64) -> (result: Ghost<Option<int>>)
        ensures
            result@ == self@.find_thread(tid as int),
    {
        Ghost(self@.find_thread(tid as int))
    }
