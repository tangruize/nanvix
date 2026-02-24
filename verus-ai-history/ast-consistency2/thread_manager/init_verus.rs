pub fn init() -> (result: (ReadyThread, ThreadManager))
    ensures
        result.0.spec_id() == 0,
        result.0.wf(),
        result.0.spec_drop_safe(),
        !result.0.spec_is_interrupted(),
        result.0.spec_locked_mutex_count() == 0,
        result.1.spec_next_id() == 1,
        result.1.wf(),
{
    ThreadManager::new()
}
