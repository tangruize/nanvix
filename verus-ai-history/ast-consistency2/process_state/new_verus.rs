    pub fn new(pid: ProcessIdentifier) -> (result: ProcessState)
        ensures
            result.spec_pid() == pid.spec_value(),
            result@.capabilities_granted =~= Set::<Capability>::empty(),
            result.spec_mutex_count() == 0,
            result.spec_cond_count() == 0,
            result.spec_pmio_count() == 0,
            result.wf(),
    {
        let caps: Capabilities = Capabilities::new();
        proof {
            caps.lemma_view_bits();
        }
        ProcessState {
            pid: pid,
            capabilities: caps,
            mutex_count: 0usize,
            mutex_addrs: Vec::new(),
            mutex_ref_counts: Vec::new(),
            cond_count: 0usize,
            cond_addrs: Vec::new(),
            cond_ref_counts: Vec::new(),
            pmio_ports: Vec::new(),
        }
    }
