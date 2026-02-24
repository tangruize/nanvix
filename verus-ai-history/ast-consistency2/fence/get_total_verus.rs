    pub fn get_total(&self) -> (result: usize)
        ensures
            result as nat == self@.total,
    {
        self.total
    }
