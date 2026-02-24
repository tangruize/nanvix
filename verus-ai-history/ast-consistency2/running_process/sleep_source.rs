    pub fn sleep(
        mut self,
        alarm: Option<SystemTime>,
    ) -> Result<
        (RunnableProcess, *mut ContextInformation),
        (SleepingProcess, *mut ContextInformation),
    > {
        let (sleeping_thread, ctx) = self.running.sleep(alarm);

        // Push sleeping thread.
        let sleeping_threads = match self.sleeping_threads.take() {
            Some(mut sleeping_threads) => {
                sleeping_threads.push_back(sleeping_thread);
                sleeping_threads
            },
            None => NonEmptyVecDeque::new(sleeping_thread),
        };

        // Check if there are ready threads.
        if let Some(ready_threads) = self.ready.take() {
            return Ok((
                RunnableProcess::from_state(
                    self.state,
                    ready_threads,
                    self.interrupted_threads.take(),
                    Some(sleeping_threads),
                    self.zombie.take(),
                ),
                ctx,
            ));
        }

        // Check if there are interrupted threads.
        if let Some(interrupted_threads) = self.interrupted_threads.take() {
            let interrupted_process: InterruptedProcess = InterruptedProcess::from_sleeping(
                self.state,
                Some(sleeping_threads),
                interrupted_threads,
                self.zombie.take(),
            );

            return Ok((interrupted_process.resume(), ctx));
        }

        Err((SleepingProcess::new(self.state, sleeping_threads, self.zombie.take()), ctx))
    }
