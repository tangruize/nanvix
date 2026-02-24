    pub unsafe fn up(&self) -> Result<(), Error> {
        self.value.fetch_add(1, Ordering::SeqCst);
        self.sleeping.notify_first().map(|_awakened| ())
    }
