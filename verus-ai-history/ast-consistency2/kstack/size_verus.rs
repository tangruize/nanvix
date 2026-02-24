    fn size(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_size(),
            // Strengthened: Size is always positive and page-aligned.
            result > 0,
            result % PAGE_SIZE == 0,
    {
        self.num_pages * PAGE_SIZE
    }
