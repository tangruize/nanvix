    pub fn get_tid(&self) -> Result<ThreadIdentifier, Error> {
        Ok(self.try_borrow()?.get_running().get_tid())
    }
