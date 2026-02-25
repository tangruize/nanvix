fn test_alloc_range_across_word_boundary_verified(number_of_bits: usize, start: usize, end: usize)
    requires
        number_of_bits >= 16,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        start < end,
        end <= number_of_bits,
        // Ensure the range crosses a byte boundary (e.g., bits 6..10).
        start / 8 < (end - 1) / 8,
{
    let result: Result<Bitmap, Error> = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set all bits that are not in the range [start, end).
        let mut i: usize = 0;
        while i < number_of_bits
            invariant
                0 <= i <= number_of_bits,
                bitmap.inv(),
                bitmap@.number_of_bits() == number_of_bits as int,
            decreases number_of_bits - i,
        {
            if i < start || i >= end {
                let _ = bitmap.set(i);
            }
            i = i + 1;
        }

        // Attempt to allocate the range.
        let size: usize = end - start;
        let alloc_result: Result<usize, Error> = bitmap.alloc_range(size);
        if let Ok(alloc_start) = alloc_result {
            // The allocated range should have all bits set.
            assert(bitmap.all_bits_set_in_range(alloc_start as int, (alloc_start + size) as int));
        }
    }
}
