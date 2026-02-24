    unsafe fn unlock_unchecked(&self) -> Result<(), Error> {
        self.locked.store(false, Ordering::Relaxed);
        self.sleeping.notify_first().map(|_awakened| ())
    }
