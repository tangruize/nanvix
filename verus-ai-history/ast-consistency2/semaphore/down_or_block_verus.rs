    pub fn down_or_block(&mut self, ctx: Ghost<CallerContext>) -> (result: DownOutcome)
        requires
            old(self).wf(),
            ctx@.safe_for_down(),
        ensures
            result == DownOutcome::Acquired ==> old(self).spec_is_available(),
            result == DownOutcome::Acquired ==> self@.value == old(self)@.value - 1,
            result == DownOutcome::Acquired ==> self@.waiters == old(self)@.waiters,
            result == DownOutcome::Acquired ==> self@ == Semaphore::spec_down_or_block_ghost_view(old(self)@, result),
            result == DownOutcome::WouldBlock ==> old(self).spec_is_exhausted(),
            result == DownOutcome::WouldBlock ==> self@ == old(self)@,
            result == DownOutcome::WouldBlock ==> Semaphore::spec_down_or_block_ghost_view(old(self)@, result).waiters == old(self)@.waiters + 1,
            result == DownOutcome::WouldBlock ==> Semaphore::spec_down_or_block_ghost_view(old(self)@, result).value == 0,
            self.wf(),
    {
        proof { reveal(Semaphore::wf); }
        if self.value > 0 {
            self.value = self.value - 1;
            DownOutcome::Acquired
        } else {
            DownOutcome::WouldBlock
        }
    }
