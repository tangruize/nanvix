    pub fn create_thread(
        &mut self,
        kernel_stack: Option<int>,
        user_stack: Option<int>,
        user_tda: Option<int>,
    ) -> (result: ReadyThread)
        requires
            old(self).wf(),
            old(self)@.next_id < i32::MAX as int,
        ensures
            result.spec_id() == old(self).spec_next_id(),
            result.spec_kernel_stack() == kernel_stack,
            result.spec_user_stack() == user_stack,
            result.spec_user_tda() == user_tda,
            result.wf(),
            result.spec_drop_safe(),
            !result.spec_is_interrupted(),
            result.spec_locked_mutex_count() == 0,
            self.spec_next_id() == old(self).spec_next_id() + 1,
            self.wf(),
    {
        proof {
            reveal(ThreadManager::wf);
            reveal(ThreadManager::spec_next_id);
            reveal(ReadyThread::spec_id);
            reveal(ReadyThread::spec_kernel_stack);
            reveal(ReadyThread::spec_user_stack);
            reveal(ReadyThread::spec_user_tda);
            reveal(ReadyThread::wf);
            reveal(ReadyThread::spec_drop_safe);
            reveal(ReadyThread::spec_is_interrupted);
            reveal(ReadyThread::spec_locked_mutex_count);
        }
        let id: ThreadIdentifier = self.next_id;
        self.next_id = ThreadIdentifier::from_i32(self.next_id.into_i32() + 1);
        ReadyThread::new(id, kernel_stack, user_stack, user_tda)
    }
