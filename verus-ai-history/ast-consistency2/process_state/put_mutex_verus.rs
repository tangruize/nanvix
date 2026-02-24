    pub fn put_mutex(
        &mut self,
        mutex_addr: u64,
        contains: bool,
        ref_count_at_threshold: bool,
        idx: usize,
    ) -> (result: Result<(), Error>)
        requires
            old(self).wf(),
            contains == old(self).spec_has_mutex(mutex_addr as int),
            contains ==> (
                idx < old(self).mutex_addrs@.len()
                && old(self).mutex_addrs@[idx as int] == mutex_addr
            ),
            contains ==> (ref_count_at_threshold ==
                (old(self).mutex_ref_counts@[idx as int] as nat <= Self::MUTEX_REMOVE_THRESHOLD())),
        ensures
            result is Ok ==> {
                &&& old(self).spec_has_mutex(mutex_addr as int)
                &&& ref_count_at_threshold ==> !self.spec_has_mutex(mutex_addr as int)
                &&& ref_count_at_threshold ==>
                        self.spec_mutex_count() == old(self).spec_mutex_count() - 1
                &&& !ref_count_at_threshold ==> self.spec_has_mutex(mutex_addr as int)
                &&& !ref_count_at_threshold ==>
                        self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& forall|a: int| a != mutex_addr as int ==>
                        self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& self.wf()
            },
            result is Err ==> {
                &&& !old(self).spec_has_mutex(mutex_addr as int)
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
        if !contains {
            let reason: &'static str = "mutex not found";
            return Err(Error::new(ErrorCode::NoSuchEntry, reason));
        }

        if ref_count_at_threshold {
            // Reference count at threshold: remove the entry from both parallel Vecs.
            self.mutex_count = self.mutex_count - 1;
            self.mutex_addrs.remove(idx);
            self.mutex_ref_counts.remove(idx);
            proof {
                assert(self.mutex_addrs@ =~= old(self).mutex_addrs@.remove(idx as int));
                assert(self.mutex_ref_counts@ =~= old(self).mutex_ref_counts@.remove(idx as int));

                // Removed address no longer present: no index maps to mutex_addr.
                assert forall|i: int|
                    0 <= i < self.mutex_addrs@.len()
                    implies self.mutex_addrs@[i] as int != mutex_addr as int by {
                    if i < idx as int {
                        assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[i]);
                    } else {
                        assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[i + 1]);
                    }
                }

                // Frame: other addresses preserved.
                assert forall|a: int| a != mutex_addr as int
                    implies (self.spec_has_mutex(a) == old(self).spec_has_mutex(a)) by {
                    assert forall|i: int|
                        0 <= i < old(self).mutex_addrs@.len() && old(self).mutex_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < self.mutex_addrs@.len() && self.mutex_addrs@[j] as int == a) by {
                        if i < idx as int {
                            assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[i]);
                        } else {
                            // i > idx (i != idx because a != mutex_addr)
                            assert(self.mutex_addrs@[i - 1] == old(self).mutex_addrs@[i]);
                        }
                    }
                    assert forall|i: int|
                        0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < old(self).mutex_addrs@.len() && old(self).mutex_addrs@[j] as int == a) by {
                        if i < idx as int {
                            assert(old(self).mutex_addrs@[i] == self.mutex_addrs@[i]);
                        } else {
                            assert(old(self).mutex_addrs@[i + 1] == self.mutex_addrs@[i]);
                        }
                    }
                }

                // wf: uniqueness preserved after remove.
                assert forall|i: int, j: int|
                    0 <= i < self.mutex_addrs@.len() && 0 <= j < self.mutex_addrs@.len() && i != j
                    implies self.mutex_addrs@[i] != self.mutex_addrs@[j] by {
                    let oi: int = if i < idx as int { i } else { i + 1 };
                    let oj: int = if j < idx as int { j } else { j + 1 };
                    assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[oi]);
                    assert(self.mutex_addrs@[j] == old(self).mutex_addrs@[oj]);
                    assert(oi != oj);
                }

                // wf: ref counts positive after remove.
                assert forall|i: int| #![auto]
                    0 <= i < self.mutex_ref_counts@.len()
                    implies self.mutex_ref_counts@[i] > 0u64 by {
                    let oi: int = if i < idx as int { i } else { i + 1 };
                    assert(self.mutex_ref_counts@[i] == old(self).mutex_ref_counts@[oi]);
                }
            }
        }
        // else: ref count above threshold — entry remains, no state change.
        Ok(())
    }
