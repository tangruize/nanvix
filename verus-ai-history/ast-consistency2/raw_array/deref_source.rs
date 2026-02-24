    fn deref(&self) -> &Self::Target {
        self.storage.get()
    }
