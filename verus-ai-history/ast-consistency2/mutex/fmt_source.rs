    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MutexGuard {{ locked: {:?}, condvar: {:?} }}",
            self.mutex.locked.load(Ordering::Relaxed),
            self.mutex.sleeping
        )
    }
