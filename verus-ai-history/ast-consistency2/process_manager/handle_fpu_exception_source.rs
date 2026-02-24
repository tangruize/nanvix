    pub fn handle_fpu_exception(&mut self) -> Result<(), Error> {
        self.try_borrow_mut()?.handle_fpu_exception()
    }
