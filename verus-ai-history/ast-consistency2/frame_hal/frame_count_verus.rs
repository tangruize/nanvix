    pub fn frame_count(&self) -> (result: usize)
        requires self.inv(),
        ensures result as int == self.spec_frame_count()
    {
        self.size / FRAME_SIZE
    }
