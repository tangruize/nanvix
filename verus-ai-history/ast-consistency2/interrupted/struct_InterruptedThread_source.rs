pub struct InterruptedThread {
    state: Box<ThreadState>,
    reason: InterruptReason,
}
