    fn new() -> (result: (ReadyThread, ThreadManager))
        ensures
            result.0.spec_id() == 0,
            result.0.wf(),
            result.0.spec_drop_safe(),
            !result.0.spec_is_interrupted(),
            result.0.spec_locked_mutex_count() == 0,
            result.1.spec_next_id() == 1,
            result.1.wf(),
    {
        proof {
            reveal(ThreadManager::wf);
            reveal(ThreadManager::spec_next_id);
            reveal(ReadyThread::spec_id);
            reveal(ReadyThread::wf);
            reveal(ReadyThread::spec_drop_safe);
            reveal(ReadyThread::spec_is_interrupted);
            reveal(ReadyThread::spec_locked_mutex_count);
        }
        let kernel: ReadyThread = ReadyThread::new(
            ThreadIdentifier::from_i32(0),
            None,
            None,
            None,
        );
        let manager: ThreadManager = ThreadManager {
            next_id: ThreadIdentifier::from_i32(1),
        };
        (kernel, manager)
    }
