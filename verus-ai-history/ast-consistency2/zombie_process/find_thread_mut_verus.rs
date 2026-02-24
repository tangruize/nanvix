    pub fn find_thread_mut(&mut self, tid: u64) -> (result: Ghost<Option<u64>>)
        requires
            old(self).inv(),
        ensures
            old(self)@.spec_has_zombie_thread(tid as int) ==> result@ == Some(0u64),
            !old(self)@.spec_has_zombie_thread(tid as int) ==> result@ == None::<u64>,
            self@ == old(self)@,
            self.inv(),
    {
        unimplemented!()
    }
