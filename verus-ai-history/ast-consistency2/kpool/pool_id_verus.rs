    pub fn pool_id(&self) -> (result: usize)
        ensures
            result as int == self@.pool_id,
            result as int == self.spec_pool_id(),
    {
        self.pool_id
    }
