unsafe fn raw_array_from_addr(addr: usize, len: usize) -> (result: Result<RawArray<u8>, Error>)
    requires
        len > 0,
        len < i32::MAX as usize,
        addr > 0,
    ensures
        result is Ok ==> {
            &&& result->Ok_0.inv()
            &&& result->Ok_0@.len() == len
            &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
        },
        result is Err ==> result->Err_0.code == ErrorCode::InvalidArgument,
{
    RawArray::from_raw_parts(addr as *mut u8, len)
}
