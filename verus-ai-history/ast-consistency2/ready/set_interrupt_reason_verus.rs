    pub fn set_interrupt_reason(&mut self, reason: int)
        requires
            old(self).wf(),
        ensures
            self.spec_is_interrupted(),
            self.spec_interrupt_reason() == Some(reason),
            self.spec_id() == old(self).spec_id(),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            self.spec_drop_safe() == old(self).spec_drop_safe(),
            self.wf(),
            self.spec_admission_time() == old(self).spec_admission_time(),
    {
        proof { reveal(ReadyThread::wf); }
        self.state.set_interrupt_reason(reason);
        proof {
            assert forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a) by {
                assert(self.state@.has_mutex(a) == old(self).state@.has_mutex(a));
            };
        }
    }
