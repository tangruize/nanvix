pub struct InterruptedProcess {
    state: Box<ProcessState>,
    sleeping_threads: Option<NonEmptyVecDeque<SleepingThread>>,
    interrupted_threads: NonEmptyVecDeque<InterruptedThread>,
    zombie_threads: Option<NonEmptyVecDeque<ZombieThread>>,
}
