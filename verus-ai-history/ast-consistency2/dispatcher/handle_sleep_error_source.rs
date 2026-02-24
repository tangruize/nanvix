fn handle_sleep_error(sleep_error: SleepError) -> KcallResult {
    match sleep_error {
        SleepError::Generic(generic_error) => {
            error!("failed to sleep: {:?}", generic_error);
            KcallResult::Error(generic_error.code.into())
        },
        SleepError::Interrupted(reason) => match reason {
            InterruptReason::Killed => {
                // SAFETY: the calling process is not the kernel.
                let error: Error =
                    unsafe { ProcessManager::exit(ErrorCode::Interrupted.into()).unwrap_err() };
                panic!("failed to exit() (error={:?})", error);
            },
            InterruptReason::TimedOut => {
                error!("failed to sleep: operation timed out");
                KcallResult::Error(ErrorCode::OperationTimedOut.into())
            },
        },
    }
}
