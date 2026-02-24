    fn take_mutex_guard(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
        mutex_addr: MutexAddress,
    ) -> Result<MutexGuard, Error> {
        let mutex_guard: MutexGuard = match self
            .get_running_mut()
            .running_mut()
            .take_mutex_guard(mutex_addr)
        {
            Some(mutex_guard) => mutex_guard,
            None => {
                let reason: &str = "thread does not own mutex";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                return Err(Error::new(ErrorCode::OperationNotPermitted, reason));
            },
        };

        self.get_running_mut().state_mut().put_mutex(mutex_addr)?;

        Ok(mutex_guard)
    }
