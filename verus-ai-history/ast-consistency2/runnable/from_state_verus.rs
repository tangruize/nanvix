    pub fn from_state(
        pid: ProcessIdentifier,
        ready_ids: Vec<i64>,
        ready_times: Vec<i64>,
        interrupted_ids: Vec<i64>,
        sleeping_ids: Vec<i64>,
        zombie_ids: Vec<i64>,
        interrupted_count: u64,
        sleeping_count: u64,
    ) -> (result: RunnableProcess)
        requires
            ready_ids@.len() >= 1,
            ready_ids@.len() == ready_times@.len(),
            forall|i: int| 0 <= i < ready_times@.len()
                ==> #[trigger] ready_times@[i] >= 0i64,
            interrupted_count as nat == interrupted_ids@.len(),
            sleeping_count as nat == sleeping_ids@.len(),
        ensures
            result@.pid == pid.spec_value(),
            result@.ready_thread_ids.len() == ready_ids@.len(),
            result@.interrupted_thread_ids.len() == interrupted_ids@.len(),
            result@.sleeping_thread_ids.len() == sleeping_ids@.len(),
            result@.zombie_thread_ids.len() == zombie_ids@.len(),
            result.wf(),
    {
        proof { reveal(RunnableProcess::wf); }
        RunnableProcess {
            pid: pid,
            ready_thread_ids: ready_ids,
            ready_admission_times: ready_times,
            interrupted_thread_ids: interrupted_ids,
            sleeping_thread_ids: sleeping_ids,
            zombie_thread_ids: zombie_ids,
            interrupted_count: interrupted_count,
            sleeping_count: sleeping_count,
        }
    }
