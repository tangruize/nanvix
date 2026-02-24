    pub fn alloc_kernel_frame(&mut self, clear: bool) -> Result<KernelFrame, Error> {
        self.kpool.alloc(clear)
    }
