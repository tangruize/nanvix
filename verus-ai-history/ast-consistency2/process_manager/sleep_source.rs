    fn sleep(
        &mut self,
        alarm: Option<SystemTime>,
    ) -> (
        ProcessIdentifier,
        ThreadIdentifier,
        *mut ContextInformation,
        *mut ContextInformation,
        Option<VirtualAddress>,
    ) {
        let running_process: RunningProcess = self.take_running();

        // Check if kernel is trying to sleep.
        if running_process.state().pid() == ProcessIdentifier::KERNEL {
            panic!("kernel process cannot sleep");
        }

        // Suspend the execution of the calling thread.
        let previous_context: *mut ContextInformation = match running_process.sleep(alarm) {
            // The calling process still has runnable threads, put it in the list of ready processes.
            Ok((runnable_process, previous_context)) => {
                self.ready.push_back(runnable_process);
                previous_context
            },
            // The calling process has only sleeping threads left, put it in the list of suspended processes.
            Err((suspended_process, previous_context)) => {
                self.suspended.push_back(suspended_process);
                previous_context
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
        (next_pid, next_tid, previous_context, next_context, user_tda)
    }
