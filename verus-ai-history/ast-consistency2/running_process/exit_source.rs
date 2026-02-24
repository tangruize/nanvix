    pub fn exit(
        mut self,
        status: ExitStatus,
    ) -> Result<(RunnableProcess, *mut ContextInformation), (ZombieProcess, *mut ContextInformation)>
    {
        let (zombie_thread, ctx) = self.running.exit(status);
        let mut zombie_threads: NonEmptyVecDeque<ZombieThread> = match self.zombie.take() {
            Some(mut zombie_threads) => {
                zombie_threads.push_back(zombie_thread);
                zombie_threads
            },
            None => NonEmptyVecDeque::new(zombie_thread),
        };

        // Terminate all ready threads.
        if let Some(ready_threads) = self.ready.take() {
            let more_zombie_threads = NonEmptyVecDeque::map(ready_threads, ReadyThread::terminate);
            zombie_threads.append(more_zombie_threads);
        }

        // Collect interrupted threads.
        let mut interrupted_threads: Option<NonEmptyVecDeque<InterruptedThread>> =
            self.interrupted_threads.take();

        // Terminate all sleeping threads.
        if let Some(sleeping_threads) = self.sleeping_threads.take() {
            let more_interrupted_threads = NonEmptyVecDeque::map(sleeping_threads, interrupt);
            match interrupted_threads.as_mut() {
                None => interrupted_threads = Some(more_interrupted_threads),
                Some(interrupted_threads) => interrupted_threads.append(more_interrupted_threads),
            }
        }

        if let Some(interrupted_threads) = interrupted_threads {
            let interrupted_process: InterruptedProcess = InterruptedProcess::from_sleeping(
                self.state,
                self.sleeping_threads.take(),
                interrupted_threads,
                Some(zombie_threads),
            );

            Ok((interrupted_process.resume(), ctx))
        } else {
            Err((ZombieProcess::new(self.state, zombie_threads, status), ctx))
        }
    }
