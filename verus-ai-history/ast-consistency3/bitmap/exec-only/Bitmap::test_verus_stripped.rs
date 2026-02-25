    pub fn test(&self, index: usize) -> Result<bool, Error>
    {
        let (word, bit): (usize, usize) = self.index(index)?;
        let byte_val: u8 = self.bits[word];
        let result_val: bool = (byte_val & (1 << bit)) != 0;

        Ok(result_val)
    }
