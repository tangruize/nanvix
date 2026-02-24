    pub fn free(&mut self, frame: FrameAddress) -> Result<(), Error> {
        let frame_number: usize = frame.into_frame_number().into_raw_value();
        match self.bitmap.clear(frame_number) {
            Ok(()) => Ok(()),
            Err(error) => {
                error!("{error:?} (frame={frame:?})");
                Err(error)
            },
        }
    }
