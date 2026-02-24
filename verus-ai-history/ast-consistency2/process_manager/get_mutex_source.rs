    fn get_mutex(&mut self, mutex_addr: MutexAddress) -> Result<Mutex, Error> {
        self.get_running_mut().state_mut().get_mutex(mutex_addr)
    }
