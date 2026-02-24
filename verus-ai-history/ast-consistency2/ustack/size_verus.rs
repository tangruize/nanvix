    pub fn size(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_size(),
            result == USER_STACK_SIZE,
            result % PAGE_SIZE == 0,
    {
        USER_STACK_SIZE
    }
