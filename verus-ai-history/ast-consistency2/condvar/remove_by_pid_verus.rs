    pub fn remove_by_pid(&mut self, pid_val: i32, idx: usize) -> (removed: bool)
        requires
            old(self).wf(),
            (idx as int) < old(self)@.sleeping.len() as int,
            old(self)@.sleeping[idx as int].0 == pid_val as int,
            // The index is the first match, modeling `position()` semantics.
            forall|k: int|
                #![trigger old(self)@.sleeping[k]]
                0 <= k < idx as int ==> old(self)@.sleeping[k].0 != pid_val as int,
        ensures
            removed,
            self@.spec_len() == old(self)@.spec_len() - 1,
            old(self)@.sleeping[idx as int].0 == pid_val as int,
            self@.sleeping =~= CondvarView::spec_remove_at_seq(old(self)@.sleeping, idx as int),
            self.wf(),
    {
        self.remove_at(idx)
    }
