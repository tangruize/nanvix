    pub fn resume(mut self) -> RunnableProcess {
        let (interrupted_threads, next_thread): (VecDeque<InterruptedThread>, InterruptedThread) =
            self.interrupted_threads.pop_front();
        let ready_thread = next_thread.resume();

        RunnableProcess::from_state(
            self.state,
            NonEmptyVecDeque::new(ready_thread),
            NonEmptyVecDeque::from(interrupted_threads),
            self.sleeping_threads.take(),
            self.zombie_threads.take(),
        )
    }
