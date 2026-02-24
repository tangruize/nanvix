    pub fn find_thread_mut(&mut self, tid: u64) -> (result: Ghost<Option<int>>)
        requires
            old(self).wf(),
        ensures
            result@ == old(self)@.spec_find_thread(tid as int),
            self@.pid == old(self)@.pid,
            self@.sleeping_thread_ids =~= old(self)@.sleeping_thread_ids,
            self@.zombie_thread_ids =~= old(self)@.zombie_thread_ids,
            self.wf(),
    {
        proof {
            reveal(SleepingProcess::wf);
            // Bridge: exec-level spec_find_thread ↔ view-level spec_find_thread.
            Self::lemma_find_thread_view_equiv(self, tid);
        }
        Ghost(old(self).spec_find_thread(tid))
    }
