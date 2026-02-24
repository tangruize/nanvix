    pub fn unlock(&mut self, Tracked(token): Tracked<LockToken>)
        requires
            old(self)@.locked,
            old(self).inv(),
            old(self)@.token_issued,
            token.view == old(self)@,
        ensures
            old(self)@.is_locked(),
            !self@.locked,
            self@.is_unlocked(),
            self@.id == old(self)@.id,
            !self@.token_issued,
            self@ == SpinlockView::spec_new(old(self)@.id),
            self.inv(),
    {
        proof { reveal(Spinlock::inv); }
        self.locked = false;
        self.token_issued = false;
    }
