    pub fn top_raw(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_top(),
            spec_is_page_aligned(result as int),
            result as int > self.spec_base(),
    {
        self.base_addr + USER_STACK_SIZE
    }
