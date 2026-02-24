pub fn load(index: usize) -> Result<usize, Error> {
    // Check if the index is out of bounds.
    if index >= KREDZONE_SIZE / mem::size_of::<usize>() {
        let reason: &str = "index out of bounds";
        error!("index={:?}, (error={})", index, reason);
        return Err(Error::new(ErrorCode::InvalidArgument, reason));
    }

    // Load the value from the kernel red zone.
    // Safety: the kernel red zone is a global static variable and index is valid.
    unsafe {
        let ptr: *const usize = core::ptr::addr_of!(kredzone);
        let ptr: *const usize = ptr.add(index);
        Ok(ptr.read_volatile())
    }
}
