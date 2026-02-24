    fn base(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_base(),
            spec_is_page_aligned(result as int),
    {
        self.base_addr
    }
