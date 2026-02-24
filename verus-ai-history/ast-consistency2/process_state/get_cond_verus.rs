    pub fn get_cond(&mut self, cond_addr: u64, already_present: bool, idx: usize) -> (result: Result<u64, Error>)
        requires
            old(self).wf(),
            already_present == old(self).spec_has_cond(cond_addr as int),
            already_present ==> (
                idx < old(self).cond_addrs@.len()
                && old(self).cond_addrs@[idx as int] == cond_addr
                && old(self).cond_ref_counts@[idx as int] < u64::MAX
            ),
        ensures
            result is Ok ==> {
                &&& self.spec_has_cond(cond_addr as int)
                &&& already_present ==>
                        self.spec_cond_ref_count(cond_addr as int)
                            == old(self).spec_cond_ref_count(cond_addr as int) + 1
                &&& !already_present ==> self.spec_cond_ref_count(cond_addr as int) == 2
                &&& result->Ok_0 as nat == self.spec_cond_ref_count(cond_addr as int)
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| a != cond_addr as int ==>
                        self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
            result is Err ==> {
                &&& result->Err_0.code == ErrorCode::OutOfMemory
                &&& old(self).spec_conditions_full()
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
        if self.cond_count >= Self::COND_MAX_EXEC() {
            let reason: &'static str = "maximum number of condition variables reached";
            return Err(Error::new(ErrorCode::OutOfMemory, reason));
        }

        if already_present {
            let old_rc: u64 = self.cond_ref_counts.remove(idx);
            self.cond_ref_counts.insert(idx, old_rc + 1);
            proof {
                assert(self.cond_ref_counts@ =~= old(self).cond_ref_counts@.update(idx as int, (old_rc + 1) as u64));
                assert(self.cond_addrs@ =~= old(self).cond_addrs@);

                // Witness for spec_has_cond.
                assert(self.cond_addrs@[idx as int] as int == cond_addr as int);

                // Uniqueness → choose picks idx.
                assert forall|i: int|
                    0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == cond_addr as int
                    implies i == idx as int by {}

                let chosen: int = choose|i: int|
                    0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == cond_addr as int;
                assert(chosen == idx as int);
                assert(self.cond_ref_counts@[chosen] as nat == (old_rc + 1) as nat);

                let old_chosen: int = choose|i: int|
                    0 <= i < old(self).cond_addrs@.len()
                    && old(self).cond_addrs@[i] as int == cond_addr as int;
                assert(old_chosen == idx as int);
                assert(old(self).cond_ref_counts@[old_chosen] as nat == old_rc as nat);

                assert forall|i: int| #![auto]
                    0 <= i < self.cond_ref_counts@.len()
                    implies self.cond_ref_counts@[i] > 0u64 by {
                    if i == idx as int {} else {}
                }
            }
            Ok(old_rc + 1)
        } else {
            self.cond_count = self.cond_count + 1;
            self.cond_addrs.push(cond_addr);
            self.cond_ref_counts.push(2);
            proof {
                assert(self.cond_addrs@ =~= old(self).cond_addrs@.push(cond_addr));
                assert(self.cond_ref_counts@ =~= old(self).cond_ref_counts@.push(2u64));

                let new_idx: int = self.cond_addrs@.len() - 1;
                assert(self.cond_addrs@[new_idx] as int == cond_addr as int);

                assert forall|i: int| 0 <= i < old(self).cond_addrs@.len()
                    implies old(self).cond_addrs@[i] as int != cond_addr as int by {}

                assert forall|i: int|
                    0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == cond_addr as int
                    implies i == new_idx by {
                    if i < old(self).cond_addrs@.len() as int {
                        assert(self.cond_addrs@[i] == old(self).cond_addrs@[i]);
                    }
                }

                let chosen: int = choose|i: int|
                    0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == cond_addr as int;
                assert(chosen == new_idx);
                assert(self.cond_ref_counts@[chosen] == 2u64);

                // Frame: other addresses preserved.
                assert forall|a: int| a != cond_addr as int
                    implies (self.spec_has_cond(a) == old(self).spec_has_cond(a)) by {
                    assert forall|i: int|
                        0 <= i < old(self).cond_addrs@.len() && old(self).cond_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < self.cond_addrs@.len() && self.cond_addrs@[j] as int == a) by {
                        assert(self.cond_addrs@[i] == old(self).cond_addrs@[i]);
                    }
                    assert forall|i: int|
                        0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < old(self).cond_addrs@.len() && old(self).cond_addrs@[j] as int == a) by {
                        if i < old(self).cond_addrs@.len() as int {
                            assert(old(self).cond_addrs@[i] == self.cond_addrs@[i]);
                        }
                    }
                }

                // wf: uniqueness after push.
                assert forall|i: int, j: int|
                    0 <= i < self.cond_addrs@.len() && 0 <= j < self.cond_addrs@.len() && i != j
                    implies self.cond_addrs@[i] != self.cond_addrs@[j] by {
                    if i < old(self).cond_addrs@.len() as int && j < old(self).cond_addrs@.len() as int {
                        assert(self.cond_addrs@[i] == old(self).cond_addrs@[i]);
                        assert(self.cond_addrs@[j] == old(self).cond_addrs@[j]);
                    } else if i == new_idx {
                        assert(self.cond_addrs@[j] == old(self).cond_addrs@[j]);
                    } else {
                        assert(self.cond_addrs@[i] == old(self).cond_addrs@[i]);
                    }
                }

                assert forall|i: int| #![auto]
                    0 <= i < self.cond_ref_counts@.len()
                    implies self.cond_ref_counts@[i] > 0u64 by {
                    if i < old(self).cond_ref_counts@.len() as int {
                        assert(self.cond_ref_counts@[i] == old(self).cond_ref_counts@[i]);
                    }
                }
            }
            Ok(2)
        }
    }
