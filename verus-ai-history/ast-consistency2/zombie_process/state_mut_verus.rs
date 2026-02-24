    pub fn state_mut(&mut self) -> (result: u64)
        requires
            old(self).inv(),
        ensures
            result as int == self@.pid,
            self@ == old(self)@,
            self.inv(),
    {
        unimplemented!()
    }
