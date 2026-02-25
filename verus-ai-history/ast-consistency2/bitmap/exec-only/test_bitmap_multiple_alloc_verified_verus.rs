fn test_bitmap_multiple_alloc_verified(number_of_bits: usize)
    requires
        number_of_bits >= 16,  // Need at least 2 bits.
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result: Result<Bitmap, Error> = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc1: Result<usize, Error> = bitmap.alloc();
        if let Ok(index1) = alloc1 {
            let alloc2: Result<usize, Error> = bitmap.alloc();
            if let Ok(index2) = alloc2 {
                // The two allocated indices should be different.
                assert(index1 != index2);
                // Both bits should be set.
                assert(bitmap.is_bit_set(index1 as int));
                assert(bitmap.is_bit_set(index2 as int));
            }
        }
    }
}
