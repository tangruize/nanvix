    pub(super) fn store_thread_data_area(&mut self, user_tda: Option<VirtualAddress>) {
        self.user_tda = user_tda;
    }
