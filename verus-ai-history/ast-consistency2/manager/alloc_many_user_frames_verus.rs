    pub fn alloc_many_user_frames(&mut self, nframes: usize) -> (result: Ghost<Seq<int>>)
        requires
            old(self).inv(),
            nframes > 0,
            old(self)@.has_upool_capacity_for(nframes as int),
        ensures
            self.inv(),
            self@.upool_free_count == old(self)@.upool_free_count - nframes as int,
    {
        self.upool.alloc_many(nframes)
    }
