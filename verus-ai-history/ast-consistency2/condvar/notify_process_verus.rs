    pub fn notify_process(&mut self, pid_val: i32, has_match: bool, match_idx: usize)
        requires
            old(self).wf(),
            has_match ==> (
                (match_idx as int) < old(self)@.sleeping.len() as int
                && old(self)@.sleeping[match_idx as int].0 == pid_val as int
                && forall|k: int|
                    #![trigger old(self)@.sleeping[k]]
                    0 <= k < match_idx as int
                    ==> old(self)@.sleeping[k].0 != pid_val as int
            ),
            !has_match ==> !old(self)@.spec_contains_pid(pid_val as int),
        ensures
            self.wf(),
            has_match ==> self@.spec_len() == old(self)@.spec_len() - 1,
            !has_match ==> self@ == old(self)@,
    {
        let _found: bool = self.try_remove_by_pid(pid_val, has_match, match_idx);
    }
