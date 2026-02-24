    pub fn notify_thread(&mut self, tid_val: i32, has_match: bool, match_idx: usize)
        requires
            old(self).wf(),
            has_match ==> (
                (match_idx as int) < old(self)@.sleeping.len() as int
                && old(self)@.sleeping[match_idx as int].1 == tid_val as int
                && forall|k: int|
                    #![trigger old(self)@.sleeping[k]]
                    0 <= k < match_idx as int
                    ==> old(self)@.sleeping[k].1 != tid_val as int
            ),
            !has_match ==> !old(self)@.spec_contains_tid(tid_val as int),
        ensures
            self.wf(),
            has_match ==> self@.spec_len() == old(self)@.spec_len() - 1,
            !has_match ==> self@ == old(self)@,
    {
        let _found: bool = self.try_remove_by_tid(tid_val, has_match, match_idx);
    }
