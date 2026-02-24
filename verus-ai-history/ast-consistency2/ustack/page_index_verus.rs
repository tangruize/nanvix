    pub fn page_index(&self, addr: usize) -> (result: usize)
        requires
            self.inv(),
            self@.contains_addr(addr as int),
        ensures
            // Precise computation: result equals the floor division of offset by page size.
            result as int == (addr as int - self.spec_base()) / (PAGE_SIZE as int),
            // Upper bound: result is within valid page range.
            (result as int) < USER_STACK_PAGES as int,
            // Semantic property: the address is within the computed page.
            self@.addr_in_page(addr as int, result as int),
    {
        (addr - self.base_addr) / PAGE_SIZE
    }
