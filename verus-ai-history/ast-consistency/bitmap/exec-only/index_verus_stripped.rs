    fn index(&self, bit_index: usize) -> Result<(usize, usize), Error>
    {
        // Check if the index is out of bounds.
        if bit_index >= self.bits.len() * u8::BITS as usize {
            let reason: &str = "index out of bounds";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }
        Ok(self.index_unchecked(bit_index))
    }
