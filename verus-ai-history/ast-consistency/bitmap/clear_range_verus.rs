    pub fn clear_range(&mut self, start: usize, size: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            size > 0,
            start as int + (size as int) <= old(self)@.number_of_bits(),
            old(self).all_bits_set_in_range(start as int, start as int + (size as int)),
        ensures
            self.inv(),
            result is Ok ==> {
                &&& self.all_bits_unset_in_range(start as int, start as int + (size as int))
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                // Frame.
                &&& forall|i: int| 0 <= i < self@.number_of_bits() &&
                    (i < start as int || i >= start as int + (size as int)) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame.
                &&& self@.set_bits =~= old(self)@.set_bits.difference(BitmapView::range_set(start as int, start as int + (size as int)))
                &&& self@.usage() == old(self)@.usage() - (size as int)
            },
            result is Err ==> self@ == old(self)@,
            result is Ok,
    {
        let ghost old_self = *self;

        // Check bounds.
        if start > self.number_of_bits - size {
            proof {
                // Unreachable due to preconditions.
                assert(start as int + (size as int) <= old(self)@.number_of_bits());
                assert(start as int <= self.number_of_bits as int - (size as int));
            }
            let reason: &str = "range out of bounds";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Clear the range one bit at a time.
        let mut offset: usize = 0;

        while offset < size
            invariant
                self.inv(),
                old_self.inv(),
                old_self == *old(self),
                self.number_of_bits == old_self.number_of_bits,
                0 < size <= self.number_of_bits,
                start <= self.number_of_bits - size,
                offset <= size,
                // Bits [start, start+offset) are cleared.
                forall|i: int| start as int <= i < (start + offset) as int ==>
                    !#[trigger] self.is_bit_set(i),
                // Bits [start+offset, start+size) are still set.
                forall|i: int| (start + offset) as int <= i < (start + size) as int ==>
                    #[trigger] self.is_bit_set(i),
                // Bits outside [start, start+size) are unchanged.
                forall|i: int| (0 <= i < self@.number_of_bits() &&
                    (i < start as int || i >= (start + size) as int)) ==>
                    #[trigger] self.is_bit_set(i) == #[trigger] old_self.is_bit_set(i),
                // Set-based invariant.
                self@.set_bits =~= old_self@.set_bits.difference(BitmapView::range_set(start as int, start as int + (offset as int))),
                // Usage tracking.
                self@.usage() == old_self@.usage() - offset,
        {
            let idx: usize = start + offset;
            let ghost loop_old_self = *self;

            proof {
                assert(self.is_bit_set(idx as int));
            }

            let clear_result: Result<(), Error> = self.clear(idx);

            proof {
                // clear succeeds because bit is set and in bounds.
                match clear_result {
                    Ok(_) => {},
                    Err(_) => {
                        // This should be unreachable.
                        assert(self.is_bit_set(idx as int));
                        assert((idx as int) < old(self)@.number_of_bits());
                    }
                }

                // Update invariants.
                assert forall|i: int| start as int <= i < (start + offset + 1) as int
                    implies !#[trigger] self.is_bit_set(i)
                by {
                    if i < (start + offset) as int {
                        // From loop invariant.
                    } else {
                        assert(i == idx as int);
                    }
                };

                // Prove set_bits invariant.
                assert forall|i: int| self@.set_bits.contains(i) ==
                    old_self@.set_bits.difference(BitmapView::range_set(start as int, start as int + (offset as int + 1))).contains(i)
                by {}
            }

            offset = offset + 1;
        }

        Ok(())
    }
