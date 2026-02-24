    pub fn contains(&self, addr: usize) -> (result: bool)
        requires
            self.inv(),
        ensures
            result == self@.contains_addr(addr as int),
    {
        addr >= self.base_addr && addr < self.base_addr + self.num_pages * PAGE_SIZE
    }
