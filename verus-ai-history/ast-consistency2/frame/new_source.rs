    pub fn new(bitmap: Bitmap) -> Self {
        let frame_allocator: FrameAllocator = Self { bitmap };

        info!(
            "frame allocator capacity: {} frames, {} MB",
            frame_allocator.bitmap.number_of_bits(),
            frame_allocator.bitmap.number_of_bits() * mem::FRAME_SIZE / constants::MEGABYTE
        );

        frame_allocator
    }
