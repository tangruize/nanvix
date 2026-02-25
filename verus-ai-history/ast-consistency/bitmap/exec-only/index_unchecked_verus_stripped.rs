    fn index_unchecked(&self, bit_index: usize) -> (usize, usize)
    {
        let word: usize = bit_index / u8::BITS as usize;
        let bit: usize = bit_index % u8::BITS as usize;
        (word, bit)
    }
