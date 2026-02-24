pub struct RunnableProcess {
    state: Box<ProcessState>,
    ready_threads: NonEmptyVecDeque<ReadyThread>,
    interrupted_threads: Option<NonEmptyVecDeque<InterruptedThread>>,
    sleeping_threads: Option<NonEmptyVecDeque<SleepingThread>>,
    zombie_threads: Option<NonEmptyVecDeque<ZombieThread>>,
}
