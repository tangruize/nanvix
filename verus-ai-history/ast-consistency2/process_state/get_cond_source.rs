    pub fn get_cond(&mut self, cond_addr: ConditionAddress) -> Result<Condvar, Error> {
        // Check if maximum number of condition variables has been reached.
        if self.conditions.len() >= COND_OPEN_MAX {
            let reason: &'static str = "maximum number of condition variables reached";
            error!("{:?} (addr={:#x?})", reason, cond_addr);
            return Err(Error::new(ErrorCode::OutOfMemory, reason));
        }

        Ok(self
            .conditions
            .entry(cond_addr)
            .or_insert_with(Condvar::new)
            .clone())
    }
