    pub fn from_state(state: ThreadState, reason: int) -> (result: InterruptedThread)
        requires
            state.wf(),
            InterruptedThreadView::spec_valid_reason(reason),
        ensures
            result@.spec_id() == state.spec_id(),
            result@.spec_reason() == reason,
            result.wf(),
    {
        proof { reveal(InterruptedThread::wf); }
        InterruptedThread { state: state, reason: reason }
    }
