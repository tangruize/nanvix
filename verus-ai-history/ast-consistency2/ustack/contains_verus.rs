    pub fn contains(&self, addr: usize) -> (result: bool)
        requires
            self.inv(),
        ensures
            result == self@.contains_addr(addr as int),
    {
        addr >= self.base_addr && addr < self.base_addr + USER_STACK_SIZE
    }
