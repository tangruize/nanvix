    pub fn find_thread(&self, tid: u64) -> (result: Ghost<Option<int>>)
        requires
            self.wf(),
        ensures
            result@ == self@.spec_find_thread(tid as int),
    {
        proof {
            Self::lemma_find_thread_view_equiv(self, tid);
        }
        Ghost(self.spec_find_thread(tid))
    }
