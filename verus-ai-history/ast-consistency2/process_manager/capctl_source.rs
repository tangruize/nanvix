    pub fn capctl(
        &mut self,
        pid: ProcessIdentifier,
        capability: Capability,
        value: bool,
    ) -> Result<(), Error> {
        self.try_borrow_mut()?.capctl(pid, capability, value)
    }
