    pub fn alloc_kpage(&mut self, clear: bool) -> Result<KernelPage, Error> {
        let kframe: KernelFrame = match self.physman.try_borrow_mut() {
            Ok(mut physman) => physman.alloc_kernel_frame(clear)?,
            Err(_) => {
                let reason: &str = "failed to borrow physical memory manager";
                error!("{reason}");
                return Err(Error::new(ErrorCode::ResourceBusy, reason));
            },
        };
        Ok(KernelPage::new(kframe))
    }
