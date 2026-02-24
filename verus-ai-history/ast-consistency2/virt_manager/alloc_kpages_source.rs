    pub fn alloc_kpages(&mut self, clear: bool, count: usize) -> Result<Vec<KernelPage>, Error> {
        let mut kpages: Vec<KernelFrame> = match self.physman.try_borrow_mut() {
            Ok(mut physman) => physman.alloc_many_kernel_frames(clear, count)?,
            Err(_) => {
                let reason: &str = "failed to borrow physical memory manager";
                error!("{reason}");
                return Err(Error::new(ErrorCode::ResourceBusy, reason));
            },
        };

        let mut pages: Vec<KernelPage> = Vec::new();
        while let Some(kframes) = kpages.pop() {
            pages.push(KernelPage::new(kframes));
        }

        Ok(pages)
    }
