pub unsafe fn unlock_mutex(
    pid: ProcessIdentifier,
    tid: ThreadIdentifier,
    mutex_addr: usize,
) -> Result<(), Error> {
    trace!("pid={pid:?}, tid={tid:?}, mutex_addr={mutex_addr:?}");

    // Unpack kernel call arguments.
    let mutex_addr: MutexAddress = MutexAddress::from(mutex_addr);

    ProcessManager::take_mutex_guard(pid, tid, mutex_addr)?;
    // The mutex guard is dropped, causing threads to be notified.

    Ok(())
}
