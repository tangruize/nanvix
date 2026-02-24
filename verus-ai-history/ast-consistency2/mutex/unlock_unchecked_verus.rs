    fn unlock_unchecked(&mut self)
        requires
            old(self)@.locked,
            old(self).wf(),
            old(self)@.token_issued,
        ensures
            !self@.locked,
            self@.is_unlocked(),
            self@.id == old(self)@.id,
            !self@.token_issued,
            self@ == MutexView::spec_new(old(self)@.id),
            self.wf(),
    {
        proof { reveal(Mutex::wf); }
        self.locked = false;
        self.token_issued = false;
    }
