    pub fn alloc_range(
        &mut self,
        region: &TruncatedMemoryRegion<PhysicalAddress>,
    ) -> Result<(), Error> {
        let start_frame_number: usize = region.start().into_frame_number().into_raw_value();
        let end_frame_number: usize = start_frame_number + region.size() / mem::FRAME_SIZE - 1;

        // Check if all frames in the range are free.
        for index in start_frame_number..=end_frame_number {
            match self.bitmap.test(index) {
                Ok(false) => continue,
                Ok(true) => {
                    return Err(Error::new(ErrorCode::OutOfMemory, "frame is already allocated"));
                },
                Err(err) => return Err(err),
            }
        }

        // Book all frames in the range.
        for index in start_frame_number..=end_frame_number {
            if let Err(error) = self.bitmap.set(index) {
                error!("{error:?} (region={region:?})");
                return Err(error);
            }
        }

        Ok(())
    }
