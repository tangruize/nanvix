    pub fn wakeup_alarm(mut self, now: SystemTime) -> Result<InterruptedProcess, SleepingProcess> {
        let (mut sleeping_threads_to_process, sleeping_thread) = self.sleeping_threads.pop_front();

        // Check if thread has an alarm set.
        if let Some(alarm) = sleeping_thread.alarm() {
            // Check if alarm has expired.
            if now >= alarm {
                let interrupt_thread: InterruptedThread =
                    sleeping_thread.interrupt(InterruptReason::TimedOut);
                let mut interrupted_threads: NonEmptyVecDeque<InterruptedThread> =
                    NonEmptyVecDeque::new(interrupt_thread);
                let mut sleeping_threads: VecDeque<SleepingThread> = VecDeque::new();

                // Process all sleeping threads.
                while let Some(sleeping_thread) = sleeping_threads_to_process.pop_front() {
                    if let Some(alarm) = sleeping_thread.alarm() {
                        if now >= alarm {
                            interrupted_threads
                                .push_back(sleeping_thread.interrupt(InterruptReason::TimedOut));
                        } else {
                            sleeping_threads.push_back(sleeping_thread);
                        }
                    } else {
                        sleeping_threads.push_back(sleeping_thread);
                    }
                }

                return Ok(InterruptedProcess::from_sleeping(
                    self.state,
                    NonEmptyVecDeque::from(sleeping_threads),
                    interrupted_threads,
                    self.zombie_threads.take(),
                ));
            } else {
                // Alarm has not expired, fallthrough.
            }
        } else {
            // Thread does not have an alarm set, fallthrough.
        }

        let mut interrupted_threads: VecDeque<InterruptedThread> = VecDeque::new();
        let mut sleeping_threads: NonEmptyVecDeque<SleepingThread> =
            NonEmptyVecDeque::new(sleeping_thread);

        // Process all sleeping threads.
        while let Some(sleeping_thread) = sleeping_threads_to_process.pop_front() {
            if let Some(alarm) = sleeping_thread.alarm() {
                if now >= alarm {
                    interrupted_threads
                        .push_back(sleeping_thread.interrupt(InterruptReason::TimedOut));
                } else {
                    sleeping_threads.push_back(sleeping_thread);
                }
            } else {
                sleeping_threads.push_back(sleeping_thread);
            }
        }

        if let Some(interrupted_threads) = NonEmptyVecDeque::from(interrupted_threads) {
            Ok(InterruptedProcess::from_sleeping(
                self.state,
                Some(sleeping_threads),
                interrupted_threads,
                self.zombie_threads.take(),
            ))
        } else {
            self.sleeping_threads = sleeping_threads;
            Err(self)
        }
    }
