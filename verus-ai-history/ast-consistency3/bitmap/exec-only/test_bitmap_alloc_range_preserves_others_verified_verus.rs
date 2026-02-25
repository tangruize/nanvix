fn test_bitmap_alloc_range_preserves_others_verified(number_of_bits: usize, size: usize, test_index: usize)
    requires
        number_of_bits >= 16,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        size > 0,
        size < number_of_bits,
        test_index < number_of_bits,
{
    let result: Result<Bitmap, Error> = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set a bit first.
        let set_result: Result<(), Error> = bitmap.set(test_index);
        if let Ok(()) = set_result {
            // Allocate a range.
            let alloc_result: Result<usize, Error> = bitmap.alloc_range(size);
            if let Ok(start_index) = alloc_result {
                // If test_index is outside the allocated range, it should still be set.
                if test_index < start_index || test_index >= start_index + size {
                    assert(bitmap.is_bit_set(test_index as int));
                }
            }
        }
    }
}
