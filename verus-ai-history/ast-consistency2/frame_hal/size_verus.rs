    pub fn size(&self) -> (result: usize)
        requires self.inv(),
        ensures result as int == self.spec_size()
    {
        self.size
    }
