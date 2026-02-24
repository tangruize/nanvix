    pub fn remove_event(&mut self, ev: &Event) -> Result<(), Error> {
        self.try_borrow_mut()?
            .get_running_mut()
            .state_mut()
            .remove_event(ev);

        Ok(())
    }
