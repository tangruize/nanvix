    pub fn free_user_frame(&mut self, frame: UserFrame) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            frame.spec_is_aligned(),
            frame.spec_frame_number() < old(self)@.upool_capacity,
            old(self).spec_uframe_is_allocated(frame.spec_raw_address()),
        ensures
            self.inv(),
    {
        self.upool.free(frame)
    }
