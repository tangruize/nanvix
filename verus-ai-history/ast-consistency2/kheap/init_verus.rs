pub unsafe fn init(addr: usize, size: usize) -> (result: Result<Kheap, Error>)
    requires
        addr > 0,
        addr % PAGE_SIZE as usize == 0,
        size >= MIN_HEAP_SIZE as usize,
        size % MIN_HEAP_SIZE as usize == 0,
        size % NUM_OF_SLABS == 0,
        (size / NUM_OF_SLABS) < i32::MAX as usize,
        (addr as int) + (size as int) <= (usize::MAX as int),
        // Alignment: size is multiple of 8 * 4096 for slab alignment.
        (size as int) % (8int * 4096int) == 0,
        // Power-of-two requirements.
        Slab::spec_is_power_of_two(8),
        Slab::spec_is_power_of_two(16),
        Slab::spec_is_power_of_two(32),
        Slab::spec_is_power_of_two(64),
        Slab::spec_is_power_of_two(128),
        Slab::spec_is_power_of_two(256),
        Slab::spec_is_power_of_two(512),
        Slab::spec_is_power_of_two(4096),
    ensures
        result is Ok ==> {
            let heap = result->Ok_0;
            &&& heap.inv()
            // Liveness: newly initialized heap is empty and ready for allocations.
            &&& heap@.is_empty()
        },
{
    Kheap::from_raw_parts(addr, size)
}
