    fn try_wakeup(&mut self, tid: ThreadIdentifier) -> Option<RunnableProcess> {
        // Search for the process in the list of sleeping processes.
        let mut suspended: LinkedList<SleepingProcess> = LinkedList::new();
        while let Some(process) = self.suspended.pop_front() {
            // Found.
            if process.find_thread(tid).is_some() {
                match process.wakeup(tid) {
                    Ok(runnable_process) => {
                        while let Some(process) = suspended.pop_back() {
                            self.suspended.push_front(process);
                        }
                        return Some(runnable_process);
                    },
                    Err(suspended_process) => {
                        self.suspended.push_front(suspended_process);
                        while let Some(process) = suspended.pop_back() {
                            self.suspended.push_front(process);
                        }
                        return None;
                    },
                }
            } else {
                suspended.push_back(process)
            }
        }
        // Process is not in the list of sleeping processes, rollback list to its original state.
        self.suspended = suspended;

        // Search for the process in the list of ready processes.
        let mut ready: LinkedList<RunnableProcess> = LinkedList::new();
        while let Some(process) = self.ready.pop_front() {
            // Found.
            if process.find_thread(tid).is_some() {
                match process.wakeup(tid) {
                    Ok(runnable_process) => {
                        while let Some(process) = ready.pop_back() {
                            self.ready.push_front(process);
                        }
                        return Some(runnable_process);
                    },
                    Err(ready_process) => {
                        self.ready.push_front(ready_process);
                        while let Some(process) = ready.pop_back() {
                            self.ready.push_front(process);
                        }
                        return None;
                    },
                }
            } else {
                ready.push_back(process)
            }
        }
        // Process is not in the list of ready processes, rollback list to its original state.
        self.ready = ready;

        None
    }
