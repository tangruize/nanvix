pub fn store(index: usize, value: usize) -> Result<(), Error> {
    // Check if the index is out of bounds.
    if index >= KREDZONE_SIZE / mem::size_of::<usize>() {
        let reason: &str = "index out of bounds";
        error!("index={:?}, value={:?}, (error={})", index, value, reason);
        return Err(Error::new(ErrorCode::InvalidArgument, reason));
    }

    // Store the value in the kernel red zone.
    // Safety: the kernel red zone is a global static variable and index is valid.
    unsafe {
        let ptr: *mut usize = core::ptr::addr_of_mut!(kredzone);
        let ptr: *mut usize = ptr.add(index);
        ptr.write_volatile(value);
    }

    Ok(())
}
