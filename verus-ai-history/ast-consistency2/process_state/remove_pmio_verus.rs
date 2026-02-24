    pub fn remove_pmio(
        &mut self,
        port_number: u16,
        found: bool,
        found_idx: usize,
    ) -> (result: Result<(), Error>)
        requires
            old(self).wf(),
            found == old(self).spec_has_pmio(port_number as int),
            found ==> (found_idx as int) < old(self).pmio_ports@.len()
                && old(self).pmio_ports@[found_idx as int] == port_number
                && forall|j: int| 0 <= j < found_idx as int ==>
                    old(self).pmio_ports@[j] as int != port_number as int,
        ensures
            result is Ok ==> {
                &&& old(self).spec_has_pmio(port_number as int)
                &&& old(self).pmio_ports@[found_idx as int] == port_number
                &&& self.spec_pmio_count() == old(self).spec_pmio_count() - 1
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
            result is Err ==> {
                &&& !old(self).spec_has_pmio(port_number as int)
                &&& result->Err_0.code == ErrorCode::NoSuchEntry
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
    {
        if !found {
            let reason: &'static str = "io port not found";
            return Err(Error::new(ErrorCode::NoSuchEntry, reason));
        }

        // Remove the element at found_idx.
        self.pmio_ports.remove(found_idx);
        Ok(())
    }
