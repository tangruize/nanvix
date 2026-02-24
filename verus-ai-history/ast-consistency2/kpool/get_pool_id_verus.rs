    pub fn get_pool_id(&self) -> (result: usize)
        ensures result as int == self@.id()
    {
        self.pool_id
    }
