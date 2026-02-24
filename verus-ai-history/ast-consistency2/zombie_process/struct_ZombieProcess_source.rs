pub struct ZombieProcess {
    zombie_threads: NonEmptyVecDeque<ZombieThread>,
    process: Box<ProcessState>,
    status: ExitStatus,
}
