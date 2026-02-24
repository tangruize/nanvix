    pub fn store_mutex_guard(&mut self, address: u64)
        requires
            old(self).wf(),
            old(self).spec_locked_mutex_count() < usize::MAX as nat,
            !old(self).spec_has_mutex(address as int),
        ensures
            self.spec_has_mutex(address as int),
            forall|a: int| a != address as int ==>
                self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count() + 1,
            !self.spec_drop_safe(),
            self.spec_id() == old(self).spec_id(),
            self.wf(),
            self.spec_admission_time() == old(self).spec_admission_time(),
    {
        proof {
            reveal(ReadyThread::wf);
        }
        self.state.store_mutex_guard(address);
    }
