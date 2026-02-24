    pub fn from_state(state: ThreadState, alarm: Option<int>) -> (result: SleepingThread)
        requires
            state.wf(),
            alarm.is_some() ==> alarm.unwrap() >= 0,
        ensures
            result.spec_id() == state.spec_id(),
            result.spec_alarm() == alarm,
            result.spec_interrupt_reason() == state.spec_interrupt_reason(),
            result.spec_user_tda() == state.spec_user_tda(),
            result.spec_kernel_stack() == state.spec_kernel_stack(),
            result.spec_user_stack() == state.spec_user_stack(),
            result.spec_locked_mutex_count() == state.spec_locked_mutex_count(),
            forall|a: int| result.spec_has_mutex(a) == state.spec_has_mutex(a),
            result.spec_drop_safe() == state.spec_drop_safe(),
            result.wf(),
    {
        SleepingThread { state: state, alarm: alarm }
    }
