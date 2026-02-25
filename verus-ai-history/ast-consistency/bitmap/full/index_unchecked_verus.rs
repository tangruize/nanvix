    fn index_unchecked(&self, bit_index: usize) -> (result: (usize, usize))
        requires
            bit_index < self.bits@.len() * u8::BITS as usize,
        ensures
            result.0 < self.bits@.len(),
            result.1 < u8::BITS as usize,
            result.0 as int == bit_index as int / (u8::BITS as int),
            result.1 as int == bit_index as int % (u8::BITS as int),
    {
        let word: usize = bit_index / u8::BITS as usize;
        let bit: usize = bit_index % u8::BITS as usize;
        (word, bit)
    }
