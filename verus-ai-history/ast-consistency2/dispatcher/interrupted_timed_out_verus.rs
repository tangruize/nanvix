    pub fn interrupted_timed_out() -> (result: SleepError)
        ensures
            result.spec_kind() =~= SleepErrorKind::InterruptedTimedOut,
            result.wf(),
    {
        SleepError { kind: SleepErrorKind::InterruptedTimedOut, error_code: 0 }
    }
