    pub fn new(pid: ProcessIdentifier, vmem: Vmem) -> Self {
        Self {
            pid,
            capabilities: Capabilities::default(),
            vmem,
            events: LinkedList::new(),
            mailbox: Mailbox::default(),
            mmio: LinkedList::new(),
            pmio: LinkedList::new(),
            mutexes: BTreeMap::new(),
            conditions: BTreeMap::new(),
        }
    }
