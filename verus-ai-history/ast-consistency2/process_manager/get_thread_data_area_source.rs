    pub fn get_thread_data_area(
        &self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
    ) -> Result<Option<VirtualAddress>, Error> {
        self.try_borrow()?.get_thread_data_area(pid, tid)
    }
