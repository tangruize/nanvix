    pub unsafe fn dispatch(
        &mut self,
        number: u32,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
        arg0: u32,
        arg1: u32,
        arg2: u32,
        arg3: u32,
    ) -> Result<KcallResult, SleepError> {
        let _guard: MutexGuard = self.lock.lock(None)?;
        self.args = KcallArgs {
            pid,
            tid,
            arg0,
            arg1,
            arg2,
            arg3,
            number,
        };
        self.dispatched.up().map_err(SleepError::Generic)?;
        self.handled.down()?;

        Ok(self.ret)
    }
