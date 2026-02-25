    pub fn log(&self) {
        eprintln!("error: {:?}: {}", self.code, self.reason);
    }
