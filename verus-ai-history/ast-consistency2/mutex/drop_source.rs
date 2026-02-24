    fn drop(&mut self) {
        // Safety: The lock is ensured to be held by the caller.
        if let Err(error) = unsafe { self.mutex.unlock_unchecked() } {
            warn!("failed to unlock mutex (self={self:?}, error={error:?})");
        }
    }
