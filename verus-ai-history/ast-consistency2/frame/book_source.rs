    pub fn book(&mut self, phys_addr: PageAligned<PhysicalAddress>) -> Result<(), Error> {
        let frame_number: usize = phys_addr.into_frame_number().into_raw_value();
        match self.bitmap.set(frame_number) {
            Ok(()) => Ok(()),
            Err(error) => {
                error!("{error:?} (phys_addr={phys_addr:?})");
                Err(error)
            },
        }
    }
