fn test_alloc_in_partial_bitmap_verified(number_of_bits: usize, set_index: usize)
    requires
        number_of_bits >= 16,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        set_index < number_of_bits,
{
    let result: Result<Bitmap, Error> = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set one bit to partially fill the bitmap.
        let set_result: Result<(), Error> = bitmap.set(set_index);
        if let Ok(()) = set_result {
            // The set bit should be marked as set.
            assert(bitmap.is_bit_set(set_index as int));

            // Allocate a new bit.
            let alloc_result: Result<usize, Error> = bitmap.alloc();
            if let Ok(index) = alloc_result {
                // The allocated bit should be set.
                assert(bitmap.is_bit_set(index as int));

                // Clear the allocated bit.
                let clear_result: Result<(), Error> = bitmap.clear(index);
                if let Ok(()) = clear_result {
                    assert(!bitmap.is_bit_set(index as int));
                    // The originally set bit should still be set (if it wasn't the one we allocated).
                    if index != set_index {
                        assert(bitmap.is_bit_set(set_index as int));
                    }
                }
            }
        }
    }
}
