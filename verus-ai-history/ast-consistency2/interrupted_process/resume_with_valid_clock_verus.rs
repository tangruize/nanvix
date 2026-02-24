    pub fn resume_with_valid_clock(
        self, admission_time: u64, clock_state: Ghost<int>,
    ) -> (result: RunnableProcess)
        requires
            self.wf(),
            Self::spec_admission_time_valid(admission_time as int, clock_state@),
        ensures
            result@.pid == self@.pid,
            result.wf(),
            result@.ready_thread_ids.len() == 1,
            result@.ready_thread_ids[0] == self@.interrupted_thread_ids[0],
            result@.ready_admission_times.len() == 1,
            result@.ready_admission_times[0] == admission_time as int,
            Self::spec_admission_time_valid(
                result@.ready_admission_times[0], clock_state@),
            result@.interrupted_thread_ids =~=
                self@.interrupted_thread_ids.subrange(
                    1, self@.interrupted_thread_ids.len() as int),
            result@.sleeping_thread_ids =~= self@.sleeping_thread_ids,
            result@.zombie_thread_ids =~= self@.zombie_thread_ids,
    {
        self.resume(admission_time)
    }
