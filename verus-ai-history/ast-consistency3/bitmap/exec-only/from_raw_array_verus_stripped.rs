    pub fn from_raw_array(array: RawArray<u8>) -> Self
    {
        let result = Self {
            number_of_bits: array.len() * u8::BITS as usize,
            bits: array,
            usage: 0,
        };
        result
    }
