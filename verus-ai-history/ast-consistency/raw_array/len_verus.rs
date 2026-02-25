    pub fn len(&self) -> (result: usize)
        ensures
            result == self@.len(),
    {
        self.storage.storage_len()
    }
