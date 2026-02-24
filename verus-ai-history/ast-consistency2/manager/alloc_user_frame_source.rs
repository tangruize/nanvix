    pub fn alloc_user_frame(&mut self) -> Result<UserFrame, Error> {
        self.upool.alloc()
    }
