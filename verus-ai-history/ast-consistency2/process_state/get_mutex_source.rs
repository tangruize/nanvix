    pub fn get_mutex(&mut self, mutex_addr: MutexAddress) -> Result<Mutex, Error> {
        // Check if maximum number of mutexes has been reached.
        if self.mutexes.len() >= MUTEX_OPEN_MAX {
            let reason: &'static str = "maximum number of mutexes reached";
            error!("{:?} (addr={:#x?})", reason, mutex_addr);
            return Err(Error::new(ErrorCode::OutOfMemory, reason));
        }

        Ok(self
            .mutexes
            .entry(mutex_addr)
            .or_insert_with(Mutex::new)
            .clone())
    }
