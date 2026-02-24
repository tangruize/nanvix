    pub fn from_state(state: ThreadState, status: int) -> (result: ZombieThread)
        requires
            state.wf(),
        ensures
            result@.state == state@,
            result@.spec_id() == state@.id,
            result@.spec_status() == status,
            result.wf(),
    {
        proof { reveal(ZombieThread::wf); }
        ZombieThread { status: status, state: state }
    }
