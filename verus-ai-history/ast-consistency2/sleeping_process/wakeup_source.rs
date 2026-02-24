    pub fn wakeup(mut self, tid: ThreadIdentifier) -> Result<RunnableProcess, SleepingProcess> {
        let sleeping_threads: NonEmptyVecDeque<SleepingThread> = self.sleeping_threads;

        // Search for the sleeping thread.
        match sleeping_threads.remove_if(|thread| thread.id() == tid) {
            Ok((sleeping_threads, sleeping_thread)) => {
                let ready_thread: ReadyThread = sleeping_thread.wakeup();
                Ok(RunnableProcess::from_state(
                    self.state,
                    NonEmptyVecDeque::new(ready_thread),
                    None,
                    NonEmptyVecDeque::from(sleeping_threads),
                    self.zombie_threads.take(),
                ))
            },
            Err(sleeping_threads) => {
                self.sleeping_threads = sleeping_threads;
                Err(self)
            },
        }
    }
