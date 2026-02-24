    pub unsafe fn notify_first(&self) -> Result<u32, Error> {
        let mut awakened: u32 = 0;

        // Attempt to wake up the first thread in the sleeping queue.
        if let Some((_pid, tid)) = self.inner.sleeping.borrow_mut().pop_front() {
            ProcessManager::wakeup(tid)?;
            awakened += 1;
        }

        Ok(awakened)
    }
