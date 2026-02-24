    pub fn put_mutex(&mut self, mutex_addr: MutexAddress) -> Result<(), Error> {
        // Check if mutex exists.
        if !self.mutexes.contains_key(&mutex_addr) {
            let reason: &'static str = "mutex not found";
            error!("{:?} (addr={:#x?})", reason, mutex_addr);
            return Err(Error::new(ErrorCode::NoSuchEntry, reason));
        }

        let _: BTreeMap<_, _> = self
            .mutexes
            .extract_if(.., |&addr, mutex| mutex_addr == addr && mutex.reference_count() <= 2)
            .collect();

        Ok(())
    }
