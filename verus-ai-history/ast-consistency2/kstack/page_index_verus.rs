    pub fn page_index(&self, addr: usize) -> (result: usize)
        requires
            self.inv(),
            self@.contains_addr(addr as int),
        ensures
            // Strengthened: Removed trivial `0 <= result as int` (always true for usize).
            // Added tighter bounds with spec functions.
            (result as int) < self.spec_num_pages(),
            result <= MAX_STACK_PAGES,
            self@.addr_in_page(addr as int, result as int),
    {
        (addr - self.base_addr) / PAGE_SIZE
    }
