    pub fn new(number_of_bits: usize) -> (result: Result<Self, Error>)
        ensures
            result is Ok ==> {
                let bitmap = result->Ok_0;
                &&& bitmap.inv()
                &&& bitmap@.number_of_bits() == number_of_bits as int
                &&& bitmap@.is_empty()
                &&& forall|i: int| 0 <= i < bitmap@.number_of_bits() ==> !bitmap.is_bit_set(i)
            },
            (number_of_bits == 0 ||
             number_of_bits >= u32::MAX as usize ||
             number_of_bits % (u8::BITS as usize) != 0) ==> result is Err,
    {
        // Check if the length is invalid.
        if number_of_bits == 0 || number_of_bits >= u32::MAX as usize {
            let reason: &str = "invalid length";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if the length is not a multiple of the number of the bitmap word.
        if number_of_bits % u8::BITS as usize != 0 {
            let reason: &str = "length must be a multiple of 8";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Allocate the bitmap.
        let mut array: RawArray<u8> = RawArray::new(number_of_bits / u8::BITS as usize)?;

        // Zero out the bitmap (restored from original source for exec consistency).
        let ghost array_len: nat = array@.len();
        let mut i: usize = 0;
        while i < array.len()
            invariant
                array@.len() == array_len,
                array_len > 0,
                array_len < i32::MAX as nat,
                array_len == number_of_bits as int / (u8::BITS as int),
                i <= array@.len(),
                forall|j: int| 0 <= j < i as int ==> array@[j] == 0u8,
                forall|j: int| i as int <= j < array@.len() as int ==> is_zero(array@[j]),
            decreases
                array@.len() - i,
        {
            array.set(i, 0u8);
            i = i + 1;
        }

        let result = Self {
            number_of_bits,
            bits: array,
            usage: 0,
        };

        proof {
            assert forall|i: int| 0 <= i < result.bits@.len() implies (result.bits@[i] == 0) by {};
            result.lemma_zero_bytes_means_empty_set();
            Self::lemma_empty_set_finite();
        }

        Ok(result)
    }
