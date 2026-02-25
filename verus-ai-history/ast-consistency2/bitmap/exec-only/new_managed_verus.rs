    pub fn new_managed(number_of_bits: usize) -> (result: Result<Self, Error>)
        requires
            number_of_bits > 0,
            number_of_bits <= (usize::MAX as int - 7) / 8 * 8,
            number_of_bits % (u8::BITS as usize) == 0,
            number_of_bits < u32::MAX as usize,
        ensures
            result is Ok ==> {
                let bmp = result->Ok_0;
                &&& bmp.inv()
                &&& bmp@.number_of_bits() == number_of_bits as int
                &&& bmp@.is_empty()
                &&& forall|i: int| 0 <= i < number_of_bits as int ==> !bmp.is_bit_set(i)
            },
    {
        Self::new(number_of_bits)
    }
