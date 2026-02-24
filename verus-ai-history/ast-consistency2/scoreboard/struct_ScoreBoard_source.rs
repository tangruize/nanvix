struct ScoreBoard {
    lock: Mutex,
    dispatched: Semaphore,
    handled: Semaphore,
    args: KcallArgs,
    ret: KcallResult,
}
