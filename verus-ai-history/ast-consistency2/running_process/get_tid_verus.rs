    pub fn get_tid(&self) -> (result: u64)
        ensures
            result as int == self@.running_thread_id,
    {
        self.running_thread_id
    }
