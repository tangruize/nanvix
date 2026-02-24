    pub unsafe fn wait(&self, alarm: Option<SystemTime>) -> Result<(), SleepError> {
        let pid: ProcessIdentifier = unsafe { ProcessManager::get() }
            .get_pid()
            .map_err(SleepError::Generic)?;

        // Check if the kernel process is trying to sleep.
        if pid == ProcessIdentifier::KERNEL {
            panic!("kernel process cannot sleep");
        }

        let tid: ThreadIdentifier = unsafe { ProcessManager::get() }
            .get_tid()
            .map_err(SleepError::Generic)?;

        // Check if alarm has already expired.
        if let Some(alarm) = alarm {
            let now: SystemTime = clock::now();
            if now >= alarm {
                error!(
                    "wait(): alarm has already expired (pid={:?}, tid={:?}, now={:?}, alarm={:?})",
                    pid, tid, now, alarm
                );
                return Err(SleepError::Generic(Error::new(
                    ErrorCode::OperationTimedOut,
                    "alarm has already expired",
                )));
            }
        }

        self.inner.sleeping.borrow_mut().push_back((pid, tid));

        match ProcessManager::sleep(alarm) {
            Ok(()) => Ok(()),
            Err(error) => {
                // Remove the thread from the sleeping queue if it was not woken up.
                self.inner
                    .sleeping
                    .borrow_mut()
                    .retain(|&mut (p, t)| p != pid || t != tid);
                Err(error)
            },
        }
    }
