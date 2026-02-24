    pub fn lock(&self) -> SpinlockGuard {
        loop {
            match self
                .0
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(false) => break,
                _ => ::arch::cpu::pause(),
            }
        }

        SpinlockGuard(self)
    }
