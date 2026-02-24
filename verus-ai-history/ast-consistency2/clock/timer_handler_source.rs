pub unsafe fn timer_handler(_intnum: InterruptNumber) {
    TIMER_TICKS.increment();

    // Check if a pause has been requested.
    #[cfg(feature = "microvm")]
    if core::ptr::read_volatile(
        ::config::microvm::DEFAULT_MICROVM_CTRL_PAUSE_REQUESTED as *const u32,
    ) == ::config::microvm::PAUSE_REQUEST
    {
        // Cause a VM exit.
        ::arch::io::out32(
            ::config::microvm::DEFAULT_VMM_PORT,
            (::config::microvm::DEFAULT_VMM_PAUSE_CMD as u32) << 16,
        )
    }

    // Determine if a context switch is required. The kernel's running state is checked first to
    // prevent reentrant calls to the scheduler, which could lead to undefined behavior.
    if !ProcessManager::is_kernel_running() {
        if let Err(error) = ProcessManager::giveup() {
            error!("context switch failed: {:?}", error);
        }
    }
}
