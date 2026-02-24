    pub unsafe fn notify_thread(&self, tid: ThreadIdentifier) -> Result<(), Error> {
        // Find thread.
        let idx: Option<usize> = self
            .inner
            .sleeping
            .borrow()
            .iter()
            .position(|&(_p, t)| t == tid);

        // Remove thread from sleeping queue.
        if let Some(at) = idx {
            let (_notified_pid, notified_tid): (ProcessIdentifier, ThreadIdentifier) =
                self.inner.sleeping.borrow_mut().remove(at);
            debug_assert!(
                notified_tid == tid,
                "notify_thread(): pid and tid do not match (expected: tid={:?}, got tid={:?})",
                tid,
                notified_tid
            );
            ProcessManager::wakeup(tid)?;
        }

        Ok(())
    }
