    pub fn alloc(&mut self) -> Result<UserFrame, Error> {
        let addr: FrameAddress = self.inner.borrow_mut().alloc()?;
        let uframe: UserFrame = UserFrame::new(addr);
        Ok(uframe)
    }
