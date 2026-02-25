    pub fn test(&self, index: usize) -> Result<bool, Error> {
        let (word, bit): (usize, usize) = self.index(index)?;
        Ok((self.bits[word] & (1 << bit)) != 0)
    }
