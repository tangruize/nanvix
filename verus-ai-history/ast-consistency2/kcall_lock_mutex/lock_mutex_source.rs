pub unsafe fn lock_mutex(
    pid: ProcessIdentifier,
    tid: ThreadIdentifier,
    mutex_addr: usize,
    timeout_s: usize,
    timeout_ns: usize,
) -> Result<(), SleepError> {
    trace!(
        "lock_mutex(): pid={pid:?}, tid={tid:?},  mutex_addr={mutex_addr:x?}, \
         timeout_s={timeout_s:?}, timeout_ns={timeout_ns:?}"
    );
    // Unpack kernel call arguments.
    let mutex_addr: MutexAddress = MutexAddress::from(mutex_addr);
    let timeout: Option<SystemTime> = if timeout_s == usize::MAX && timeout_ns == usize::MAX {
        None
    } else {
        match SystemTime::new(timeout_s as u64, timeout_ns as u32) {
            Some(timeout) => Some(timeout),
            None => {
                let reason: &str = "invalid timeout";
                error!(
                    "lock_mutex(): {} (mutex_addr={:x?}, timeout_s={:?}, timeout_ns={:?})",
                    reason, mutex_addr, timeout_s, timeout_ns
                );
                return Err(SleepError::Generic(Error::new(ErrorCode::InvalidArgument, reason)));
            },
        }
    };

    let mutex: Mutex = ProcessManager::get_mutex(mutex_addr).map_err(SleepError::Generic)?;
    let guard: MutexGuard = mutex.lock(timeout)?;
    ProcessManager::put_mutex_guard(mutex_addr, guard).map_err(SleepError::Generic)
}
