    pub fn remove_mmio_stub(&mut self)
        requires
            old(self).wf(),
        ensures
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        unimplemented!()
    }
