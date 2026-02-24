    pub fn up(&mut self, ctx: Ghost<CallerContext>)
        requires
            old(self).wf(),
            old(self)@.value < usize::MAX as nat,
            ctx@.safe_for_up(),
        ensures
            self@.value == old(self)@.value + 1,
            self@.waiters == old(self)@.waiters,
            self.spec_is_available(),
            self.wf(),
    {
        proof { reveal(Semaphore::wf); }
        self.value = self.value + 1;
    }
