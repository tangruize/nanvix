fn test_bitmap_number_of_bits_constant_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result: Result<Bitmap, Error> = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let ghost initial_bits: int = bitmap@.number_of_bits();
        assert(initial_bits == number_of_bits as int);

        // After allocation.
        let alloc_result: Result<usize, Error> = bitmap.alloc();
        if let Ok(_) = alloc_result {
            assert(bitmap@.number_of_bits() == initial_bits);

            // After setting a bit.
            let set_result: Result<(), Error> = bitmap.set(index);
            match set_result {
                Ok(()) => {
                    assert(bitmap@.number_of_bits() == initial_bits);
                },
                Err(_) => {
                    // If set failed (bit already set), number_of_bits should still be the same.
                    assert(bitmap@.number_of_bits() == initial_bits);
                }
            }
        }
    }
}
