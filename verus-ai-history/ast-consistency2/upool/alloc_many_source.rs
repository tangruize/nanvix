    pub fn alloc_many(&mut self, nframes: usize) -> Result<Vec<UserFrame>, Error> {
        trace!("nframes={nframes:?}");

        // Attempt to allocate pages.
        let mut uframes: Vec<FrameAddress> = self.inner.borrow_mut().alloc_many(nframes)?;

        // Create a vector of user pages.
        let mut upages: Vec<UserFrame> = Vec::new();
        while let Some(page) = uframes.pop() {
            let upage: UserFrame = UserFrame::new(page);
            upages.push(upage);
        }

        Ok(upages)
    }
