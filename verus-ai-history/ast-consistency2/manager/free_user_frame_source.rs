    pub fn free_user_frame(&mut self, frame: UserFrame) -> Result<(), Error> {
        self.upool.free(frame)
    }
