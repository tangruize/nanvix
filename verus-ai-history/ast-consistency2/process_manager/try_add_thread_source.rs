    fn try_add_thread(
        &mut self,
        pid: ProcessIdentifier,
        ready_thread: ReadyThread,
    ) -> ThreadIdentifier {
        trace!("pid={pid:?}, ready_thread={ready_thread:?}");
        let tid: ThreadIdentifier = ready_thread.id();

        // Search process in the list of sleeping processes.
        let mut suspended: LinkedList<SleepingProcess> = LinkedList::new();
        while let Some(process) = self.suspended.pop_front() {
            // Found.
            if process.state().pid() == pid {
                let ready_process: RunnableProcess = process.add_thread(ready_thread);
                // Rollback list to its original state.
                while let Some(process) = suspended.pop_back() {
                    self.suspended.push_front(process);
                }
                // Push process to the list of ready processes.
                self.ready.push_back(ready_process);
                return tid;
            }
            suspended.push_back(process);
        }
        // Process is not in the list of sleeping processes, rollback list to its original state.
        self.suspended = suspended;

        // Search process in the list of ready processes.
        let mut ready: LinkedList<RunnableProcess> = LinkedList::new();
        while let Some(process) = self.ready.pop_front() {
            // Found.
            if process.state().pid() == pid {
                let ready_process: RunnableProcess = process.add_thread(ready_thread);
                // Rollback list to its original state.
                while let Some(process) = ready.pop_back() {
                    self.ready.push_front(process);
                }
                // Push process to the list of ready processes.
                self.ready.push_back(ready_process);
                return tid;
            }
            ready.push_back(process);
        }
        // Process is not in the list of ready processes, rollback list to its original state.
        self.ready = ready;

        unreachable!("process must be either sleeping or runnable")
    }
