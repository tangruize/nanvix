    pub fn alloc_many_user_frames(&mut self, nframes: usize) -> Result<Vec<UserFrame>, Error> {
        self.upool.alloc_many(nframes)
    }
