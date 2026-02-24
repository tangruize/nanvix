    pub fn alloc_kpages(&mut self, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            old(self)@.has_kpool_capacity_for(count as int),
        ensures
            self.inv(),
            result.is_ok() ==> self@.kpool_free_count == old(self)@.kpool_free_count - count as int,
            result.is_err() ==> self@ == old(self)@,
    {
        unimplemented!()
    }
