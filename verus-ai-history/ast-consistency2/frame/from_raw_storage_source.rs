    pub fn from_raw_storage(storage: RawArray<u8>) -> Result<Self, Error> {
        Ok(Self::new(Bitmap::from_raw_array(storage)))
    }
