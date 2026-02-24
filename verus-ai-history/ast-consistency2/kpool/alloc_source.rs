    pub fn alloc(&mut self, clear: bool) -> Result<KernelFrame, Error> {
        let frame: FrameAddress = self.inner.borrow_mut().alloc()?;
        let mut kframe: KernelFrame = KernelFrame::new(self.inner.clone(), frame);
        if clear {
            // TODO: move clear logic to page-level.
            kframe.clear();
        }
        Ok(kframe)
    }
