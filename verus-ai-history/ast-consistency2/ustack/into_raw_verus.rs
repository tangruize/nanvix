    pub fn into_raw(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_addr(),
            spec_is_page_aligned(result as int),
    {
        self.addr
    }
