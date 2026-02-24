    pub fn alloc_many(&mut self, clear: bool, count: usize) -> Result<Vec<KernelFrame>, Error> {
        // Attempt to allocate pages.
        let mut kframes: Vec<FrameAddress> = self.inner.borrow_mut().alloc_range(count)?;

        // Create a vector of kernel pages.
        let mut kpages: Vec<KernelFrame> = Vec::new();
        while let Some(kframe) = kframes.pop() {
            let mut kframe: KernelFrame = KernelFrame::new(self.inner.clone(), kframe);
            if clear {
                // TODO: move clear logic to page-level.
                kframe.clear();
            }
            kpages.push(kframe);
        }

        Ok(kpages)
    }
