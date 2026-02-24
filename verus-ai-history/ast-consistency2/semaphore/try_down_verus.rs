    pub fn try_down(&mut self) -> (result: bool)
        requires
            old(self).wf(),
        ensures
            result == old(self).spec_is_available(),
            result ==> self@.value == old(self)@.value - 1,
            !result ==> self@ == old(self)@,
            !result ==> self.spec_is_exhausted(),
            self@.waiters == old(self)@.waiters,
            self.wf(),
    {
        proof { reveal(Semaphore::wf); }
        if self.value > 0 {
            self.value = self.value - 1;
            true
        } else {
            false
        }
    }
