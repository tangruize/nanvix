    pub fn lock(&mut self) -> (token: Tracked<MutexToken>)
        requires
            old(self)@.is_unlocked(),
            old(self).wf(),
            !old(self)@.token_issued,
        ensures
            self@.locked,
            self@.is_locked(),
            self@.id == old(self)@.id,
            self@.token_issued,
            token@.view == self@,
            self.wf(),
    {
        let (_success, Tracked(opt_token)) = self.try_lock();
        let tracked token: MutexToken = opt_token.tracked_unwrap();
        Tracked(token)
    }
