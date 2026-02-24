    pub fn try_remove_by_pid(&mut self, pid_val: i32, has_match: bool, match_idx: usize) -> (found: bool)
        requires
            old(self).wf(),
            // If match exists: match_idx is the first valid match.
            has_match ==> (
                (match_idx as int) < old(self)@.sleeping.len() as int
                && old(self)@.sleeping[match_idx as int].0 == pid_val as int
                && forall|k: int|
                    #![trigger old(self)@.sleeping[k]]
                    0 <= k < match_idx as int ==> old(self)@.sleeping[k].0 != pid_val as int
            ),
            // If no match: no entry has matching pid.
            !has_match ==> !old(self)@.spec_contains_pid(pid_val as int),
        ensures
            found == has_match,
            // If found: removal happened.
            found ==> self@.spec_len() == old(self)@.spec_len() - 1,
            found ==> self@.sleeping =~= CondvarView::spec_remove_at_seq(
                old(self)@.sleeping, match_idx as int,
            ),
            // If not found: state unchanged.
            !found ==> self@ == old(self)@,
            self.wf(),
    {
        if has_match {
            self.remove_at(match_idx);
            true
        } else {
            false
        }
    }
