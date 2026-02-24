    pub fn address(&self) -> (result: FrameAddress)
        ensures
            result == self.spec_address(),
            result.spec_frame_number() == self@.frame_number,
            result.spec_is_aligned() == self.spec_is_aligned(),
    {
        self.addr
    }
