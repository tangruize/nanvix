    pub fn has_capability(
        &self,
        pid: ProcessIdentifier,
        capability: Capability,
    ) -> Result<bool, Error> {
        Ok(self
            .try_borrow()?
            .find_process(pid)?
            .state()
            .has_capability(capability))
    }
