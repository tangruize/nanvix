    pub fn num_pages(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_num_pages(),
            // Strengthened: Guarantee positive count and bounded.
            result > 0,
            result <= MAX_STACK_PAGES,
    {
        self.num_pages
    }
