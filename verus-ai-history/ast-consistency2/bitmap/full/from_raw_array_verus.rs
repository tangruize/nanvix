    pub fn from_raw_array(array: RawArray<u8>) -> (result: Self)
        requires
            array@.len() > 0,
            array@.len() <= usize::MAX / (u8::BITS as usize),
            array@.len() * (u8::BITS as usize) < u32::MAX as usize,
            forall|i: int| 0 <= i < array@.len() ==> array@[i] == 0,
        ensures
            result.inv(),
            result@.number_of_bits() == array@.len() * (u8::BITS as int),
            result@.is_empty(),
            forall|i: int| 0 <= i < result@.number_of_bits() ==> !result.is_bit_set(i),
    {
        let result = Self {
            number_of_bits: array.len() * u8::BITS as usize,
            bits: array,
            usage: 0,
        };
        proof {
            result.lemma_zero_bytes_means_empty_set();
            Self::lemma_empty_set_finite();
        }
        result
    }
