    pub fn set_thread_data_area(&mut self, user_tda: Option<VirtualAddress>) {
        self.state.store_thread_data_area(user_tda);
    }
