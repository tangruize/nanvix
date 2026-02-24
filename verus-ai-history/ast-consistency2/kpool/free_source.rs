    fn free(&mut self, addr: FrameAddress) -> Result<(), Error> {
        let index: usize =
            (addr.into_raw_value() - self.region.start().into_raw_value()) / mem::PAGE_SIZE;
        match self.bitmap.clear(index) {
            Ok(()) => Ok(()),
            Err(error) => {
                error!("{error:?} (addr={addr:?})");
                Err(error)
            },
        }
    }
