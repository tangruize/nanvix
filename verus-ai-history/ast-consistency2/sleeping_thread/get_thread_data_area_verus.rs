    pub fn get_thread_data_area(&self) -> (result: Option<int>)
        requires
            self.wf(),
        ensures
            result == self.spec_user_tda(),
    {
        self.state.get_thread_data_area()
    }
