    pub fn bury(self) -> (result: (Vec<u64>, u64, i64))
        requires
            self.inv(),
        ensures
            Seq::new(result.0@.len(), |i: int| result.0@[i] as int) =~= self@.zombie_thread_ids,
            result.1 as int == self@.pid,
            result.2 as int == self@.status,
            result.0@.len() >= 1,
    {
        (self.zombie_thread_ids, self.pid, self.status)
    }
