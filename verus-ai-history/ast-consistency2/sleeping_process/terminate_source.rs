    pub fn terminate(self) -> InterruptedProcess {
        let (mut sleeping_threads, sleeping_thread): (VecDeque<SleepingThread>, SleepingThread) =
            self.sleeping_threads.pop_front();

        let mut interrupted_threads: NonEmptyVecDeque<InterruptedThread> =
            NonEmptyVecDeque::new(sleeping_thread.interrupt(InterruptReason::Killed));

        while let Some(sleeping_thread) = sleeping_threads.pop_front() {
            interrupted_threads.push_back(sleeping_thread.interrupt(InterruptReason::Killed));
        }

        InterruptedProcess::new(self.state, interrupted_threads, self.zombie_threads)
    }
