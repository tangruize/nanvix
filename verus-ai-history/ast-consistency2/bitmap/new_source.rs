    pub fn new(number_of_bits: usize) -> Result<Self, Error> {
        // Check if the length is invalid.
        if number_of_bits == 0 || number_of_bits >= u32::MAX as usize {
            let reason: &str = "invalid length";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if the length is not a multiple of the number of the bitmap word.
        if !number_of_bits.is_multiple_of(u8::BITS as usize) {
            let reason: &str = "length must be a multiple of 8";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Allocate the bitmap.
        let mut array: RawArray<u8> = RawArray::new(number_of_bits / u8::BITS as usize)?;

        // Zero out the bitmap.
        for byte in array.iter_mut() {
            *byte = 0;
        }

        Ok(Self {
            number_of_bits,
            bits: array,
            usage: 0,
        })
    }
