    fn check_alarm(&mut self) {
        let now: SystemTime = clock::now();

        // Create a temporary list to store processes that are still sleeping.
        let mut suspended: LinkedList<SleepingProcess> = LinkedList::new();

        // Filter out processes that are still sleeping.
        while let Some(process) = self.suspended.pop_front() {
            // Attempt to wake up process.
            match process.wakeup_alarm(now) {
                Ok(interrupted_process) => {
                    trace!(
                        "process {:?} interrupted at {now:?}",
                        interrupted_process.state().pid(),
                    );
                    self.interrupted.push_back(interrupted_process);
                },
                Err(suspended_process) => suspended.push_back(suspended_process),
            }
        }

        // Set the list of sleeping processes.
        self.suspended = suspended;
    }
