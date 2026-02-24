    pub fn unlock(&mut self, Tracked(token): Tracked<MutexToken>)
        requires
            old(self)@.locked,
            old(self).wf(),
            old(self)@.token_issued,
            token.view == old(self)@,
        ensures
            old(self)@.is_locked(),
            !self@.locked,
            self@.is_unlocked(),
            self@.id == old(self)@.id,
            !self@.token_issued,
            self@ == MutexView::spec_new(old(self)@.id),
            self.wf(),
    {
        self.unlock_unchecked();
    }
