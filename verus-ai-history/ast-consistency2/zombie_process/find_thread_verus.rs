    pub fn find_thread(&self, tid: u64) -> (result: Ghost<Option<u64>>)
        requires
            self.inv(),
        ensures
            self@.spec_has_zombie_thread(tid as int) ==> result@ == Some(0u64),
            !self@.spec_has_zombie_thread(tid as int) ==> result@ == None::<u64>,
    {
        unimplemented!()
    }
