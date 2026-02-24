pub struct SleepingProcess {
    state: Box<ProcessState>,
    sleeping_threads: NonEmptyVecDeque<SleepingThread>,
    zombie_threads: Option<NonEmptyVecDeque<ZombieThread>>,
}
