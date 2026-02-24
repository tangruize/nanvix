    pub fn number_buffered_messages(&self) -> Result<usize, Error> {
        Ok(self.try_borrow()?.number_buffered_messages)
    }
