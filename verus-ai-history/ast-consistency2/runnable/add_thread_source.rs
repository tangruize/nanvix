    pub fn add_thread(mut self, ready_thread: ReadyThread) -> Self {
        trace!("self.pid={:?}, ready_thread={:?}", self.state.pid, ready_thread);
        self.ready_threads.push_back(ready_thread);
        self
    }
