    pub fn from_raw_array(mut array: RawArray<u8>) -> Self {
        // NOTE: no need to test if the length of the raw array is valid, as it is by construction.

        // Zero out the bitmap.
        for byte in array.iter_mut() {
            *byte = 0;
        }

        Self {
            number_of_bits: array.len() * u8::BITS as usize,
            bits: array,
            usage: 0,
        }
    }
