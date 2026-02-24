    fn find_thread_mut(&mut self, tid: ThreadIdentifier) -> Result<ThreadRefMut<'_>, Error> {
        // Search thread in the running process.
        if let Some(thread) = self.running.as_mut() {
            if let Some(thread) = thread.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of ready processes.
        for process in self.ready.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of sleeping processes.
        for process in self.suspended.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of interrupted processes.
        for process in self.interrupted.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of zombie processes.
        for process in self.zombies.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        let reason: &str = "thread not found";
        error!("{reason} (tid={tid:?})");
        Err(Error::new(ErrorCode::NoSuchEntry, reason))
    }
