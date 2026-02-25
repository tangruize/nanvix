    fn test_unchecked(&self, index: usize) -> (result: bool)
        requires
            self.inv(),
            index < self.number_of_bits,
        ensures
            result == self.is_bit_set(index as int),
    {
        let (word, bit): (usize, usize) = self.index_unchecked(index);
        (self.bits[word] & (1 << bit)) != 0
    }
