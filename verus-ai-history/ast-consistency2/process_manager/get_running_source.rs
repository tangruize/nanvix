    fn get_running(&self) -> &RunningProcess {
        // NOTE: it is safe to call unwrap because there is always a process running.
        self.running.as_ref().expect("the kernel should be running")
    }
