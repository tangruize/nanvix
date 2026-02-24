    pub unsafe fn down(&self) -> Result<(), SleepError> {
        loop {
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

            self.sleeping.wait(None)?;
        }
    }
