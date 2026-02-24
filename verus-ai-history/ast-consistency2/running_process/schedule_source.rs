    pub fn schedule(mut self) -> (RunnableProcess, *mut ContextInformation) {
        let running_thread = self.running;
        let (ready_thread, ctx) = running_thread.schedule();

        let ready_threads = match self.ready.take() {
            Some(mut ready_threads) => {
                ready_threads.push_back(ready_thread);
                ready_threads
            },
            None => NonEmptyVecDeque::new(ready_thread),
        };

        (
            RunnableProcess::from_state(
                self.state,
                ready_threads,
                self.interrupted_threads.take(),
                self.sleeping_threads.take(),
                self.zombie.take(),
            ),
            ctx,
        )
    }
