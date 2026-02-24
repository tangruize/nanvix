pub struct ProcessState {
    /// Process identifier.
    pid: ProcessIdentifier,
    /// Capabilities.
    capabilities: Capabilities,
    /// Memory address space.
    vmem: Vmem,
    /// Event ownerships.
    events: LinkedList<EventOwnership>,
    /// Incoming messages.
    mailbox: Mailbox,
    /// Memory mapped I/O regions.
    mmio: LinkedList<IoMemoryRegion>,
    /// I/O ports.
    pmio: LinkedList<AnyIoPort>,
    /// Mutexes.
    mutexes: BTreeMap<MutexAddress, Mutex>,
    /// Condition variables.
    conditions: BTreeMap<ConditionAddress, Condvar>,
}
