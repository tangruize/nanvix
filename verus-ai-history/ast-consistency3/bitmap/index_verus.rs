    fn index(&self, bit_index: usize) -> (result: Result<(usize, usize), Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                &&& bit_index < self.number_of_bits
                &&& result->Ok_0.0 < self.bits@.len()
                &&& result->Ok_0.1 < u8::BITS as usize
                &&& result->Ok_0.0 as int == bit_index as int / (u8::BITS as int)
                &&& result->Ok_0.1 as int == bit_index as int % (u8::BITS as int)
            },
            result is Err ==> bit_index >= self.number_of_bits,
            bit_index < self.number_of_bits ==> result is Ok,
    {
        // Check if the index is out of bounds.
        if bit_index >= self.bits.len() * u8::BITS as usize {
            let reason: &str = "index out of bounds";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }
        Ok(self.index_unchecked(bit_index))
    }
