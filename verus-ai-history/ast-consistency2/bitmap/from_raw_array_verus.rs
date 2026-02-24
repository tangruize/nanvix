    pub fn from_raw_array(mut array: RawArray<u8>) -> (result: Self)
        requires
            array@.len() > 0,
            array@.len() <= usize::MAX / (u8::BITS as usize),
            array@.len() * (u8::BITS as usize) < u32::MAX as usize,
        ensures
            result.inv(),
            result@.number_of_bits() == array@.len() * (u8::BITS as int),
            result@.is_empty(),
            forall|i: int| 0 <= i < result@.number_of_bits() ==> !result.is_bit_set(i),
    {
        // Zero out the bitmap (restored from original source for exec consistency).
        let ghost array_len: nat = array@.len();
        let mut i: usize = 0;
        while i < array.len()
            invariant
                array@.len() == array_len,
                array_len > 0,
                array_len <= usize::MAX / (u8::BITS as usize),
                array_len * (u8::BITS as usize) < u32::MAX as usize,
                i <= array@.len(),
                forall|j: int| 0 <= j < i as int ==> array@[j] == 0u8,
            decreases
                array@.len() - i,
        {
            array.set(i, 0u8);
            i = i + 1;
        }

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
