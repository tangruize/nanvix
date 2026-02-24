pub unsafe fn join_thread(
    pid: ProcessIdentifier,
    arg0: u32,
    arg1: u32,
) -> Result<ExitStatus, SleepError> {
    // Unpack kernel call arguments.
    let tid: ThreadIdentifier = match ThreadIdentifier::try_from(arg0) {
        Ok(tid) => tid,
        Err(error) => {
            error!("{error:?}");
            return Err(SleepError::Generic(error));
        },
    };
    let retval: *mut ExitStatus = arg1 as *mut ExitStatus;

    let status: ExitStatus = ProcessManager::join_thread(pid, tid)?;

    pm::copy_to_user::<ExitStatus>(ProcessManager::get_mut(), pid, retval, &status)
        .map_err(SleepError::Generic)?;

    Ok(ExitStatus::ok())
}
