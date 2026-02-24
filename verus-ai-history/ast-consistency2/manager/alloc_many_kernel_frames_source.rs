    pub fn alloc_many_kernel_frames(
        &mut self,
        clear: bool,
        count: usize,
    ) -> Result<Vec<KernelFrame>, Error> {
        self.kpool.alloc_many(clear, count)
    }
