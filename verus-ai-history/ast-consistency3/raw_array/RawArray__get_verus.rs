    pub fn get(&self, index: usize) -> (result: &T)
        requires
            self.in_bounds(index as int),
        ensures
            *result == self@[index as int],
    {
        &self.storage.get()[index]
    }
