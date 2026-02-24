pub struct ProcessState {
    /// Process identifier (verified dependency).
    pub pid: ProcessIdentifier,
    /// Capabilities bitfield (verified dependency).
    pub capabilities: Capabilities,
    /// Number of mutexes in the map.
    pub mutex_count: usize,
    /// Mutex addresses (concrete, models BTreeMap keys).
    /// Keys are unique (enforced by wf()).
    pub mutex_addrs: Vec<u64>,
    /// Mutex reference counts (parallel to mutex_addrs).
    /// Each value models `Arc::strong_count()` for the corresponding mutex.
    pub mutex_ref_counts: Vec<u64>,
    /// Number of condition variables in the map.
    pub cond_count: usize,
    /// Condvar addresses (concrete, models BTreeMap keys).
    /// Keys are unique (enforced by wf()).
    pub cond_addrs: Vec<u64>,
    /// Condvar reference counts (parallel to cond_addrs).
    pub cond_ref_counts: Vec<u64>,
    /// I/O port numbers (concrete, models LinkedList<AnyIoPort>).
    pub pmio_ports: Vec<u16>,
}
