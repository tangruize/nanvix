    pub fn top(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_top(),
            spec_is_page_aligned(result as int),
            // Strengthened: top is strictly greater than base.
            result > self.spec_base(),
    {
        self.base_addr + self.num_pages * PAGE_SIZE
    }
