    pub fn find_thread(&self, tid: u64) -> (result: Ghost<Option<int>>)
        ensures
            result@ == self@.spec_find_thread(tid as int),
    {
        proof {
            // Bridge: exec-level spec_find_thread ↔ view-level spec_find_thread.
            Self::lemma_find_thread_view_equiv(self, tid);
        }
        Ghost(self.spec_find_thread(tid))
    }
