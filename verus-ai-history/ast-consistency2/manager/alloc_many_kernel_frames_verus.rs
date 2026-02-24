    pub fn alloc_many_kernel_frames(
        &mut self,
        _clear: bool,
        count: usize,
    ) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
            count > 0,
            count as int <= old(self)@.kpool_capacity,
        ensures
            self.inv(),
            result.is_ok() ==> self@.kpool_free_count == old(self)@.kpool_free_count - count as int,
    {
        self.kpool.alloc_many(count)
    }
