    fn schedule(
        &mut self,
    ) -> (
        ProcessIdentifier,
        ThreadIdentifier,
        *mut ContextInformation,
        *mut ContextInformation,
        Option<VirtualAddress>,
    ) {
        // Reschedule running process.
        let previous_process: RunningProcess = self.take_running();

        let (previous_process, previous_context) = previous_process.schedule();
        self.ready.push_back(previous_process);

        self.check_alarm();

        // Process all interrupted processes.
        while let Some(interrupted_process) = self.interrupted.pop_front() {
            let ready_process: RunnableProcess = interrupted_process.resume();
            self.ready.push_back(ready_process);
        }

        // Select next process to run.
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
