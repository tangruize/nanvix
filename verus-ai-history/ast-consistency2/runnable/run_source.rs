    pub fn run(
        mut self,
    ) -> (RunningProcess, Option<InterruptReason>, *mut ContextInformation, Option<VirtualAddress>)
    {
        let mut ready_threads: VecDeque<ReadyThread> = self.ready_threads.into();

        // Select thread with the earliest admission time.
        let mut index_selected_thread: usize = 0;
        for (i, thread) in ready_threads.iter().enumerate() {
            if thread.admission_time() < ready_threads[index_selected_thread].admission_time() {
                index_selected_thread = i;
            }
        }
        let next_thread: ReadyThread = match ready_threads.remove(index_selected_thread) {
            Some(thread) => thread,
            None => {
                // SAFETY: the following statement is unreachable because there should always be at
                // least one ready thread in a runnable process.
                unreachable!("no ready threads in runnable process");
            },
        };

        let (running_thread, interrupt_reason, next_context, user_tda): (
            RunningThread,
            Option<InterruptReason>,
            *mut ContextInformation,
            Option<VirtualAddress>,
        ) = next_thread.run();
        (
            RunningProcess::new(
                self.state,
                running_thread,
                NonEmptyVecDeque::from(ready_threads),
                self.interrupted_threads.take(),
                self.sleeping_threads.take(),
                self.zombie_threads.take(),
            ),
            interrupt_reason,
            next_context,
            user_tda,
        )
    }
