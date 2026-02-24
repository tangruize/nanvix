    pub fn try_join_thread(
        &mut self,
        tid: ThreadIdentifier,
    ) -> Result<ZombieThread, Result<Condvar, Error>> {
        // Check if the thread is the running thread.
        if self.running.id() == tid {
            let reason: &str = "thread is running";
            return Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)));
        }

        // Search for thread in zombie threads.
        if let Some(zombie_threads) = self.zombie.take() {
            match zombie_threads.remove_if(|thread| thread.id() == tid) {
                Ok((zombie_threads, zombie_thread)) => {
                    self.zombie = NonEmptyVecDeque::from(zombie_threads);
                    return Ok(zombie_thread);
                },
                Err(zombie_threads) => {
                    self.zombie = Some(zombie_threads);
                },
            }
        }

        // Search for thread in ready threads.
        if let Some(ready_threads) = &mut self.ready {
            for ready_thread in ready_threads.iter() {
                if ready_thread.id() == tid {
                    let join_cond: Condvar = ready_thread.join_cond();
                    return Err(Ok(join_cond));
                }
            }
        }

        // Search for thread in sleeping threads.
        if let Some(sleeping_threads) = &mut self.sleeping_threads {
            for sleeping_thread in sleeping_threads.iter() {
                if sleeping_thread.id() == tid {
                    let join_cond: Condvar = sleeping_thread.join_cond();
                    return Err(Ok(join_cond));
                }
            }
        }

        // Search for thread in interrupted threads.
        if let Some(interrupted_threads) = &mut self.interrupted_threads {
            for interrupted_thread in interrupted_threads.iter() {
                if interrupted_thread.id() == tid {
                    let join_cond: Condvar = interrupted_thread.join_cond();
                    return Err(Ok(join_cond));
                }
            }
        }

        let reason: &str = "thread not found";
        error!("{:?} (state={:?})", reason, self.state());
        Err(Err(Error::new(ErrorCode::NoSuchProcess, reason)))
    }
