    pub fn add_event(&mut self, ownership: EventOwnership) -> Result<(), Error> {
        self.try_borrow_mut()?
            .get_running_mut()
            .state_mut()
            .add_event(ownership);

        Ok(())
    }
