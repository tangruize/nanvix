    pub fn free(&mut self, uframe: UserFrame) -> Result<(), Error> {
        self.inner.borrow_mut().free(uframe.address())
    }
