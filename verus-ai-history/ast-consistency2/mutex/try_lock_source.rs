    pub fn try_lock(&self) -> Result<MutexGuard, ()> {
        if self
            .0
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            Err(())
        } else {
            Ok(MutexGuard {
                mutex: self.0.clone(),
            })
        }
    }
