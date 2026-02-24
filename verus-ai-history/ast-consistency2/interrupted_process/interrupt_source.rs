pub(super) fn interrupt(thread: SleepingThread) -> InterruptedThread {
    thread.interrupt(InterruptReason::Killed)
}
