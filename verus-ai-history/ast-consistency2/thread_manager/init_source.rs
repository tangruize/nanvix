pub fn init() -> (ReadyThread, ThreadManager) {
    // TODO: check for double initialization.

    ThreadManager::new()
}
