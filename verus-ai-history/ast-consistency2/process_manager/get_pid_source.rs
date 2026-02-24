    pub fn get_pid(&self) -> Result<ProcessIdentifier, Error> {
        // SAFETY: This is the only thread running, thus access to the process manager is synchronized.
        Ok(self.try_borrow()?.get_running().state().pid())
    }
