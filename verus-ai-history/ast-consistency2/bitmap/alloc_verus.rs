    pub fn alloc(&mut self) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                let index = result->Ok_0 as int;
                &&& 0 <= index < self@.number_of_bits()
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& self.is_bit_set(index)
                &&& !old(self).is_bit_set(index)
                &&& !old(self)@.is_full()
                // Frame: only the allocated bit changed.
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame.
                &&& self@.set_bits =~= old(self)@.set_bits.insert(index)
                &&& self@.usage() == old(self)@.usage() + 1
            },
            result is Err ==> self@ == old(self)@,
            old(self)@.has_free_bit() ==> result is Ok,
    {
        proof {
            if old(self)@.has_free_bit() {
                old(self).lemma_has_free_bit_implies_exists_free_range_1();
            }
        }
        self.alloc_range(1)
    }
