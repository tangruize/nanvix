fn test_bitmap_set_clear_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result: Result<Bitmap, Error> = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set the bit.
        let set_result: Result<(), Error> = bitmap.set(index);
        if let Ok(()) = set_result {
            // The bit should be set.
            assert(bitmap.is_bit_set(index as int));

            // Clear the bit.
            let clear_result: Result<(), Error> = bitmap.clear(index);
            if let Ok(()) = clear_result {
                // The bit should be cleared.
                assert(!bitmap.is_bit_set(index as int));
            }
        }
    }
}
