    pub fn create_thread(
        &mut self,
        mm: &mut VirtMemoryManager,
        pid: ProcessIdentifier,
        thread_create_args: &ThreadCreateArgs,
    ) -> Result<ThreadIdentifier, Error> {
        // Assert pre-conditions (these should have been checked by the caller).
        debug_assert!(Vmem::is_user_addr(thread_create_args.user_fn));

        self.try_borrow_mut()?
            .create_thread(mm, pid, thread_create_args)
    }
