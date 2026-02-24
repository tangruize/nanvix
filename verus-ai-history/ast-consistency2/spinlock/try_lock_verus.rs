    pub fn try_lock(&mut self) -> (result: (bool, Tracked<Option<LockToken>>))
        requires
            old(self).inv(),
        ensures
            result.0 == !old(self)@.locked,
            // Unconditional: lock is always held after try_lock (success: acquired;
            // failure: was already locked, state unchanged).
            self@.locked,
            self@.id == old(self)@.id,
            !result.0 ==> self@ == old(self)@,
            result.0 ==> self@.token_issued,
            result.0 ==> result.1@.is_some(),
            result.0 ==> result.1@.unwrap().view == self@,
            !result.0 ==> result.1@.is_none(),
            !result.0 ==> self@.token_issued == old(self)@.token_issued,
            self.inv(),
    {
        proof { reveal(Spinlock::inv); }
        if !self.locked {
            self.locked = true;
            self.token_issued = true;
            let tracked token: LockToken = LockToken { view: self@ };
            (true, Tracked(Some(token)))
        } else {
            (false, Tracked(None))
        }
    }
