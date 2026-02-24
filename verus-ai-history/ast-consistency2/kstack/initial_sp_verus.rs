    pub fn initial_sp(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_top(),
            // Strengthened: initial_sp is page-aligned and greater than base.
            spec_is_page_aligned(result as int),
            result > self.spec_base(),
    {
        self.top()
    }
