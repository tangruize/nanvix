fn test_bitmap_alloc_range_verified(number_of_bits: usize, size: usize)
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
        if let Ok(start_index) = alloc_result {
            // The start index should be within valid range.
            assert(start_index + size <= number_of_bits);

            // All bits in the range should be set.
            assert(bitmap.all_bits_set_in_range(start_index as int, (start_index + size) as int));
        }
    }
}
