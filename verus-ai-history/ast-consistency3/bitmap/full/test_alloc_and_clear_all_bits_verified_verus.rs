fn test_alloc_and_clear_all_bits_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result: Result<Bitmap, Error> = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Allocate all bits.
        let mut count: usize = 0;
        while count < number_of_bits
            invariant
                0 <= count <= number_of_bits,
                bitmap.inv(),
                bitmap@.number_of_bits() == number_of_bits as int,
                bitmap@.usage() == count as int,
            decreases number_of_bits - count,
        {
            let alloc_result: Result<usize, Error> = bitmap.alloc();
            if let Ok(_) = alloc_result {
                count = count + 1;
            } else {
                break;
            }
        }

        // If we allocated all bits successfully.
        if count == number_of_bits {
            // Usage should equal number_of_bits.
            assert(bitmap@.usage() == number_of_bits as int);

            // Clear all bits.
            let mut j: usize = 0;
            while j < number_of_bits
                invariant
                    0 <= j <= number_of_bits,
                    bitmap.inv(),
                    bitmap@.number_of_bits() == number_of_bits as int,
                    forall|k: int| 0 <= k < j as int ==> !bitmap.is_bit_set(k),
                decreases number_of_bits - j,
            {
                let clear_result: Result<(), Error> = bitmap.clear(j);
                if let Ok(()) = clear_result {
                    j = j + 1;
                } else {
                    break;
                }
            }

            // If we cleared all bits successfully.
            if j == number_of_bits {
                // All bits should be cleared.
                assert(bitmap.all_bits_unset_in_range(0, number_of_bits as int));
            }
        }
    }
}
