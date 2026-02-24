    fn exit_thread(
        &mut self,
        status: ExitStatus,
    ) -> (
        ProcessIdentifier,
        ThreadIdentifier,
        Condvar,
        *mut ContextInformation,
        *mut ContextInformation,
        Option<VirtualAddress>,
    ) {
        let running_process: RunningProcess = self.take_running();

        trace!(
            "pid={:?}, tid={:?}, status={:?}",
            running_process.state().pid(),
            running_process.get_tid(),
            status
        );

        // Check if kernel is trying to exit.
        if running_process.state().pid() == ProcessIdentifier::KERNEL {
            panic!("kernel process cannot exit (status={status:?})");
        }

        // Terminate the calling thread and schedule another thread to run.
        let (join_cond, previous_context): (Condvar, *mut ContextInformation) =
            match running_process.exit_thread(status) {
                // The calling process still has runnable threads, put it in the list of ready processes.
                Ok((join_cond, runnable_process, previous_context)) => {
                    self.ready.push_back(runnable_process);
                    (join_cond, previous_context)
                },
                // The calling process has only sleeping threads left, put it in the list of suspended processes.
                Err(Ok((join_cond, sleeping_process, previous_context))) => {
                    self.suspended.push_back(sleeping_process);
                    (join_cond, previous_context)
                },
                // The calling process has only zombie threads left, put it in the list of zombies processes.
                Err(Err((join_cond, zombie_process, previous_context))) => {
                    self.zombies.push_back(zombie_process);
                    (join_cond, previous_context)
                },
            };

        // Schedule another thread to run.
        let next_process: RunnableProcess = self.take_earliest_ready();

        let (next_process, reason, next_context, user_tda): (
            RunningProcess,
            Option<InterruptReason>,
            *mut ContextInformation,
            Option<VirtualAddress>,
        ) = next_process.run();

        let next_pid: ProcessIdentifier = next_process.state().pid();
        let next_tid: ThreadIdentifier = next_process.get_tid();
        self.interrupt_reason = reason;
        self.running = Some(next_process);
        (next_pid, next_tid, join_cond, previous_context, next_context, user_tda)
    }
