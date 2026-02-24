    pub fn down_available(&mut self, ctx: Ghost<CallerContext>)
        requires
            old(self).wf(),
            old(self).spec_is_available(),
            ctx@.safe_for_down(),
        ensures
            self@.value == old(self)@.value - 1,
            self@.waiters == old(self)@.waiters,
            self.wf(),
    {
        proof { reveal(Semaphore::wf); }
        self.value = self.value - 1;
    }
