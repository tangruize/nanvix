    pub unsafe fn notify_process(&self, pid: ProcessIdentifier) -> Result<(), Error> {
        // Find process.
        let idx: Option<usize> = self
            .inner
            .sleeping
            .borrow()
            .iter()
            .position(|&(p, _)| p == pid);

        // Remove process from sleeping queue.
        if let Some(at) = idx {
            let (_notified_pid, tid) = self.inner.sleeping.borrow_mut().remove(at);
            debug_assert!(
                _notified_pid == pid,
                "notify_process(): pid and tid do not match (expected: pid={:?}, got pid={:?})",
                pid,
                _notified_pid
            );
            ProcessManager::wakeup(tid)?;
        }

        Ok(())
    }
