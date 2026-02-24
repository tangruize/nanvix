pub unsafe fn sleep(seconds: usize, nanoseconds: usize) -> Result<(), SleepError> {
    trace!("seconds={seconds:?}, nanoseconds={nanoseconds:?}");

    // Get the current time.
    let now: SystemTime = clock::now();

    // Calculate the wake up time.
    let timeout: Duration = Duration::new(seconds as u64, nanoseconds as u32);
    let alarm: SystemTime = match now.checked_add_duration(&timeout) {
        Some(wakeup_time) => wakeup_time,
        None => {
            let reason: &str = "invalid sleep time";
            return Err(SleepError::Generic(Error::new(ErrorCode::InvalidArgument, reason)));
        },
    };

    match ProcessManager::sleep(Some(alarm)) {
        Ok(()) => Ok(()),
        Err(SleepError::Interrupted(InterruptReason::TimedOut)) => Ok(()),
        Err(error) => Err(error),
    }
}
