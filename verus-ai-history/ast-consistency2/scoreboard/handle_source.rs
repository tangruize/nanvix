    pub fn handle(&self) -> Result<&KcallArgs, Error> {
        self.dispatched.try_down()?;

        Ok(&self.args)
    }
