    pub fn set_thread_data_area(&mut self, user_tda: Option<int>)
        requires
            old(self).wf(),
        ensures
            self.spec_user_tda() == user_tda,
            self.spec_id() == old(self).spec_id(),
            self.spec_alarm() == old(self).spec_alarm(),
            self.spec_locked_mutex_count() == old(self).spec_locked_mutex_count(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            self.spec_drop_safe() == old(self).spec_drop_safe(),
            self.wf(),
    {
        proof { reveal(SleepingThread::wf); }
        // Capture pre-mutation drop_safe via lemma (bridges impl-level and view-level).
        let ghost pre_drop_safe: bool = self.state.spec_drop_safe();
        proof {
            self.state.lemma_check_drop_safe_models_drop();
        }
        self.state.store_thread_data_area(user_tda);
        proof {
            self.state.lemma_check_drop_safe_models_drop();
            assert forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
            by {
                assert(self.spec_has_mutex(a) == self.state@.has_mutex(a));
                assert(old(self).spec_has_mutex(a) == old(self).state@.has_mutex(a));
            }
        }
    }
