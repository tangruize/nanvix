    fn take_earliest_ready(&mut self) -> RunnableProcess {
        // SAFETY: As the kernel process is always runnable, the following statement will never panic.
        let mut selected: (usize, SystemTime) = (
            0,
            self.ready
                .front()
                .expect("there should always be a process ready to run")
                .earliest_admission_time(),
        );

        // Select process with the earliest admission time.
        for (i, process) in self.ready.iter().enumerate() {
            let process_admission_time: SystemTime = process.earliest_admission_time();
            if process_admission_time < selected.1 {
                selected = (i, process_admission_time);
            }
        }

        // Remove the selected process from the list of ready processes.
        self.ready.remove(selected.0)
    }
