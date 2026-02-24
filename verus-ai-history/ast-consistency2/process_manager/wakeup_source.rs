    pub fn wakeup(&mut self, tid: ThreadIdentifier) -> Result<(), Error> {
        // Check if thread belongs to the running process.
        if self.get_running().find_thread(tid).is_some() {
            let running_process: RunningProcess = self.take_running();
            match running_process.wakeup(tid) {
                Ok(running_process) => {
                    self.running = Some(running_process);
                    return Ok(());
                },
                Err(running_process) => {
                    self.running = Some(running_process);
                    let reason: &str = "thread not found";
                    error!("{reason} (tid={tid:?})");
                    return Err(Error::new(ErrorCode::NoSuchEntry, reason));
                },
            }
        }

        // Check if thread belongs to a suspended process.
        let runnable_process: RunnableProcess = match self.try_wakeup(tid) {
            Some(runnable_process) => runnable_process,
            None => {
                let reason: &str = "thread not found";
                error!("{reason} (tid={tid:?})");
                return Err(Error::new(ErrorCode::NoSuchEntry, reason));
            },
        };

        self.ready.push_back(runnable_process);

        Ok(())
    }
