    pub fn get_thread_data_area(&self) -> (result: Option<int>)
        ensures
            result == self@.user_tda,
    {
        self.user_tda
    }
