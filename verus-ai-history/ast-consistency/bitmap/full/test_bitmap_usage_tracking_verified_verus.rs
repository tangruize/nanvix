fn test_bitmap_usage_tracking_verified(number_of_bits: usize)
    requires
        number_of_bits >= 24,  // Need at least 3 bits.
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result: Result<Bitmap, Error> = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Initially empty.
        assert(bitmap@.is_empty());
        assert(bitmap@.usage() == 0);

        // Allocate first bit.
        let alloc1: Result<usize, Error> = bitmap.alloc();
        if let Ok(_) = alloc1 {
            assert(bitmap@.usage() == 1);

            // Allocate second bit.
            let alloc2: Result<usize, Error> = bitmap.alloc();
            if let Ok(_) = alloc2 {
                assert(bitmap@.usage() == 2);

                // Allocate third bit.
                let alloc3: Result<usize, Error> = bitmap.alloc();
                if let Ok(index3) = alloc3 {
                    assert(bitmap@.usage() == 3);

                    // Clear one bit.
                    let clear_result: Result<(), Error> = bitmap.clear(index3);
                    if let Ok(()) = clear_result {
                        assert(bitmap@.usage() == 2);
                    }
                }
            }
        }
    }
}
