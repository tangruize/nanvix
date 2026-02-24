    pub fn terminate(&mut self, pid: ProcessIdentifier) -> Result<(), Error> {
        self.try_borrow_mut()?.terminate(pid)
    }
