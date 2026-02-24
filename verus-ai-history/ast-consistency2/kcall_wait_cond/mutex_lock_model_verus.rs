pub fn mutex_lock_model(mutex_addr: u32) -> (result: LockOutcomeModel)
    ensures
        // TimedOut is impossible with None timeout.
        !matches!(result, LockOutcomeModel::TimedOut),
        result matches LockOutcomeModel::Ok
            ==> spec_mutex_reacquired(mutex_addr as nat),
{
    unimplemented!()
}
