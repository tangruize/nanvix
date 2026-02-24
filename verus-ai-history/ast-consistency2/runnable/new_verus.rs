    pub fn new(pid: ProcessIdentifier, ready_tid: i64, ready_time: i64) -> (result: RunnableProcess)
        requires
            ready_time >= 0i64,
        ensures
            result@.pid == pid.spec_value(),
            result@.ready_thread_ids.len() == 1,
            result@.interrupted_thread_ids.len() == 0,
            result@.sleeping_thread_ids.len() == 0,
            result@.zombie_thread_ids.len() == 0,
            result.wf(),
    {
        proof { reveal(RunnableProcess::wf); }
        let mut rid_vec: Vec<i64> = Vec::new();
        rid_vec.push(ready_tid);
        let mut rtime_vec: Vec<i64> = Vec::new();
        rtime_vec.push(ready_time);
        RunnableProcess {
            pid: pid,
            ready_thread_ids: rid_vec,
            ready_admission_times: rtime_vec,
            interrupted_thread_ids: Vec::new(),
            sleeping_thread_ids: Vec::new(),
            zombie_thread_ids: Vec::new(),
            interrupted_count: 0,
            sleeping_count: 0,
        }
    }
