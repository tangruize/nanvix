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

        // Allocate the bitmap (already zero-initialized by RawArray::new).
        let array: RawArray<u8> = RawArray::new(number_of_bits / u8::BITS as usize)?;

        let result = Self {
            number_of_bits,
            bits: array,
            usage: 0,
        };

        proof {
            assert forall|i: int| 0 <= i < result.bits@.len() implies (result.bits@[i] == 0) by {
                axiom_u8_zero_is_0(result.bits@[i]);
            };
            result.lemma_zero_bytes_means_empty_set();
            Self::lemma_empty_set_finite();
        }

        Ok(result)
    }
