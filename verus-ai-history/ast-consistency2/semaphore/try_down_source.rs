    pub fn try_down(&self) -> Result<(), Error> {
        if self
            .value
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                if value == 0 {
                    None
                } else {
                    Some(value - 1)
                }
            })
            .is_ok()
        {
            return Ok(());
        }

        Err(Error::new(ErrorCode::TryAgain, "semaphore is busy"))
    }
