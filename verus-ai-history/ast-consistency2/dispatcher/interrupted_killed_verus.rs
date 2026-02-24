    pub fn interrupted_killed() -> (result: SleepError)
        ensures
            result.spec_kind() =~= SleepErrorKind::InterruptedKilled,
            result.wf(),
    {
        SleepError { kind: SleepErrorKind::InterruptedKilled, error_code: 0 }
    }
