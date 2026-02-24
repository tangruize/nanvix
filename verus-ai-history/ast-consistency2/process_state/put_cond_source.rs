    pub fn put_cond(&mut self, cond_addr: ConditionAddress) -> Result<(), Error> {
        // Check if condition variable exists.
        if !self.conditions.contains_key(&cond_addr) {
            let reason: &'static str = "condition variable not found";
            error!("{:?} (addr={:#x?})", reason, cond_addr);
            return Err(Error::new(ErrorCode::NoSuchEntry, reason));
        }

        let _: BTreeMap<_, _> = self
            .conditions
            .extract_if(.., |&addr, cond| cond_addr == addr && cond.reference_count() <= 1)
            .collect();

        Ok(())
    }
