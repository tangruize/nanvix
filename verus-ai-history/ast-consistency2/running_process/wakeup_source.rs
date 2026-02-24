    pub fn wakeup(mut self, tid: ThreadIdentifier) -> Result<RunningProcess, RunningProcess> {
        if let Some(sleeping_threads) = self.sleeping_threads.take() {
            match sleeping_threads.remove_if(|thread| thread.id() == tid) {
                Ok((sleeping_threads, sleeping_thread)) => {
                    let ready_thread: ReadyThread = sleeping_thread.wakeup();

                    let ready_threads = match self.ready.take() {
                        Some(mut ready_threads) => {
                            ready_threads.push_back(ready_thread);
                            ready_threads
                        },
                        None => NonEmptyVecDeque::new(ready_thread),
                    };

                    Ok(Self::new(
                        self.state,
                        self.running,
                        Some(ready_threads),
                        self.interrupted_threads.take(),
                        NonEmptyVecDeque::from(sleeping_threads),
                        self.zombie.take(),
                    ))
                },
                Err(sleeping_threads) => {
                    self.sleeping_threads = Some(sleeping_threads);
                    Err(self)
                },
            }
        } else {
            Err(self)
        }
    }
