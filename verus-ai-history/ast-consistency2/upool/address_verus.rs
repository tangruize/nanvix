    pub fn address(&self) -> (result: FrameAddress)
        ensures result == self.spec_address()
    {
        self.addr
    }
