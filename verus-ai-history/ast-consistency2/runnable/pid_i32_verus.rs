    pub fn pid_i32(&self) -> (result: i32)
        ensures
            result as int == self@.pid,
    {
        self.pid.into_i32()
    }
