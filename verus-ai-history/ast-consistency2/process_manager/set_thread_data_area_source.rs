    pub fn set_thread_data_area(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
        user_tda: Option<VirtualAddress>,
    ) -> Result<(), Error> {
        self.try_borrow_mut()?
            .set_thread_data_area(pid, tid, user_tda)
    }
