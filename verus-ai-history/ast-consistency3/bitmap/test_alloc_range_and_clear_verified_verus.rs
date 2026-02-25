fn test_alloc_range_and_clear_verified(number_of_bits: usize, size: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        size > 0,
        size <= number_of_bits,
{
    let result: Result<Bitmap, Error> = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc_result: Result<usize, Error> = bitmap.alloc_range(size);
        if let Ok(start) = alloc_result {
            // Verify the range is allocated.
            assert(bitmap.all_bits_set_in_range(start as int, (start + size) as int));

            // Clear the range.
            let mut i: usize = start;
            let end: usize = start + size;
            while i < end
                invariant
                    start <= i <= end,
                    end == start + size,
                    bitmap.inv(),
                    bitmap@.number_of_bits() == number_of_bits as int,
                    forall|j: int| start as int <= j < i as int ==> !bitmap.is_bit_set(j),
                decreases end - i,
            {
                let clear_result: Result<(), Error> = bitmap.clear(i);
                if let Ok(()) = clear_result {
                    i = i + 1;
                } else {
                    break;
                }
            }

            // If we cleared all bits successfully.
            if i == end {
                // All bits in the range should be cleared.
                assert(bitmap.all_bits_unset_in_range(start as int, end as int));
            }
        }
    }
}
