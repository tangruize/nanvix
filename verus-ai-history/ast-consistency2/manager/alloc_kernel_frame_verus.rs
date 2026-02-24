    pub fn alloc_kernel_frame(&mut self, _clear: bool) -> (result: Result<KernelFrame, Error>)
        requires
            old(self).inv(),
            old(self)@.has_kpool_capacity(),
        ensures
            self.inv(),
            result.is_ok() ==> self@.kpool_free_count == old(self)@.kpool_free_count - 1,
    {
        self.kpool.alloc()
    }
