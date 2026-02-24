    pub fn new(bitmap: Bitmap) -> (result: FrameAllocator)
        requires
            bitmap.inv(),
            bitmap@.number_of_bits() > 0,
            bitmap@.number_of_bits() <= MAX_FRAME_NUMBER as int + 1,
            // All bits initially unset (all frames free).
            forall|i: int| 0 <= i < bitmap@.number_of_bits() ==> !bitmap.is_bit_set(i),
        ensures
            result.inv(),
            result@.capacity == bitmap@.number_of_bits(),
            result@.is_freshly_initialized(),
    {
        let frame_allocator: FrameAllocator = FrameAllocator { bitmap };

        // Prove the allocator is freshly initialized.
        proof {
            assert(frame_allocator@.allocated_frames =~= Set::<int>::empty()) by {
                assert forall|i: int| !frame_allocator@.allocated_frames.contains(i) by {
                    if 0 <= i < frame_allocator.bitmap@.number_of_bits() {
                        assert(!frame_allocator.bitmap.is_bit_set(i));
                    }
                }
            }
        }

        frame_allocator
    }
