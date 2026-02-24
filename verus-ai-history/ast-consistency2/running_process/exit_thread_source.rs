    pub fn exit_thread(
        mut self,
        status: ExitStatus,
    ) -> Result<
        (Condvar, RunnableProcess, *mut ContextInformation),
        Result<
            (Condvar, SleepingProcess, *mut ContextInformation),
            (Condvar, ZombieProcess, *mut ContextInformation),
        >,
    > {
        let join_cond: Condvar = self.running.join_cond();

        let (zombie_thread, ctx) = self.running.exit(status);
        let zombie_threads: NonEmptyVecDeque<ZombieThread> = match self.zombie.take() {
            Some(mut zombie_threads) => {
                zombie_threads.push_back(zombie_thread);
                zombie_threads
            },
            None => NonEmptyVecDeque::new(zombie_thread),
        };

        if let Some(ready_threads) = self.ready.take() {
            Ok((
                join_cond,
                RunnableProcess::from_state(
                    self.state,
                    ready_threads,
                    self.interrupted_threads.take(),
                    self.sleeping_threads.take(),
                    Some(zombie_threads),
                ),
                ctx,
            ))
        } else if let Some(interrupted_threads) = self.interrupted_threads.take() {
            let interrupted_process: InterruptedProcess = InterruptedProcess::from_sleeping(
                self.state,
                self.sleeping_threads.take(),
                interrupted_threads,
                Some(zombie_threads),
            );

            Ok((join_cond, interrupted_process.resume(), ctx))
        } else if let Some(sleeping_threads) = self.sleeping_threads.take() {
            Err(Ok((
                join_cond,
                SleepingProcess::new(self.state, sleeping_threads, Some(zombie_threads)),
                ctx,
            )))
        } else {
            Err(Err((join_cond, ZombieProcess::new(self.state, zombie_threads, status), ctx)))
        }
    }
