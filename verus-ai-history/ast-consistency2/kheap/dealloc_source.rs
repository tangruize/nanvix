    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let heap = ptr::addr_of_mut!(HEAP);
        if let Some(heap) = &mut *heap {
            if let Err(e) = heap.deallocate(ptr, layout) {
                error!("deallocation failed (layout={:?}): {:?}", layout, e);
            }
        }
    }
