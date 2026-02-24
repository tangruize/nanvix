    fn drop(&mut self) {
        if !self.locked_mutexes.is_empty() {
            error!(
                "drop(): dropping thread state with locked mutexes (self.id={:?}, \
                 self.locked_mutexes={:?})",
                self.id, self.locked_mutexes
            );
        }
    }
