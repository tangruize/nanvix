    pub fn add_thread(mut self, ready_thread: ReadyThread) -> RunnableProcess {
        trace!("self.pid={:?}, ready_thread={:?}", self.state.pid, ready_thread);
        RunnableProcess::from_state(
            self.state,
            NonEmptyVecDeque::new(ready_thread),
            None,
            Some(self.sleeping_threads),
            self.zombie_threads.take(),
        )
    }
