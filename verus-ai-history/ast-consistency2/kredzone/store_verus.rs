pub fn store(index: usize, value: usize) -> (result: Result<(), Error>)
    ensures
        // Success case: index was valid.
        result.is_ok() ==> spec_is_valid_index(index as int),
        // Failure case: index was out of bounds.
        result.is_err() ==> !spec_is_valid_index(index as int),
{
    // Verified bounds check.
    if index >= NUM_ENTRIES {
        return Err(Error::new(ErrorCode::InvalidArgument, "index out of bounds"));
    }
    
    // Trusted raw memory access.
    raw_store(index, value);
    Ok(())
}
