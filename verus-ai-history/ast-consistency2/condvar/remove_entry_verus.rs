    pub fn remove_entry(&mut self, pid_val: i32, tid_val: i32, idx: usize) -> (removed: bool)
        requires
            old(self).wf(),
            (idx as int) < old(self)@.sleeping.len() as int,
            old(self)@.sleeping[idx as int] == (pid_val as int, tid_val as int),
        ensures
            removed,
            self@.spec_len() == old(self)@.spec_len() - 1,
            self@.sleeping =~= CondvarView::spec_remove_at_seq(old(self)@.sleeping, idx as int),
            self.wf(),
    {
        self.remove_at(idx)
    }
