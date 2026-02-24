    pub unsafe fn lock(&self, timeout: Option<SystemTime>) -> Result<MutexGuard, SleepError> {
        loop {
            // Attempt to acquire the mutex.
            match self.try_lock() {
                // Success.
                Ok(guard) => break Ok(guard),
                // Failed to acquire the mutex.
                Err(()) => {
                    self.0.sleeping.wait(timeout)?;
                },
            }
        }
    }
