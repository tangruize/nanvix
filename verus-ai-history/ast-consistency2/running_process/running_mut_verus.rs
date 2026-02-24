    pub fn running_mut(&mut self) -> (result: u64)
        ensures
            result as int == self@.running_thread_id,
            // Frame: mutation through running_mut does not change modeled fields.
            self@ == old(self)@,
            self.inv() == old(self).inv(),
    {
        unimplemented!()
    }
