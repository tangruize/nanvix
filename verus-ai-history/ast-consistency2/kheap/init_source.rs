pub unsafe fn init() -> Result<(), Error> {
    info!("initializing the kernel heap...");

    HEAP = Some(Kheap::from_raw_parts(
        HEAP_STORAGE.memory.as_ptr() as usize,
        HEAP_STORAGE.memory.len(),
    )?);

    Ok(())
}
