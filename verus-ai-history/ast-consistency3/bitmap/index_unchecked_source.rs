    fn index_unchecked(&self, index: usize) -> (usize, usize) {
        let word: usize = index / u8::BITS as usize;
        let bit: usize = index % u8::BITS as usize;
        (word, bit)
    }
