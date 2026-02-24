    pub fn state_mut(&mut self) -> (result: u64)
        ensures
            result as int == self@.pid,
            // Frame: mutation through state_mut does not change modeled fields.
            self@ == old(self)@,
            self.inv() == old(self).inv(),
    {
        unimplemented!()
    }
