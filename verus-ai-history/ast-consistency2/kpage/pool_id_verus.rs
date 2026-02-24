    pub fn pool_id(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.pool_id(),
    {
        self.kframe.pool_id()
    }
