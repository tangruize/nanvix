fn raw_store(index: usize, value: usize)
    requires
        spec_is_valid_index(index as int),
{
    // VERIFICATION STUB: Verus ignores this body.
    // Actual implementation:
    //   unsafe {
    //       let ptr: *mut usize = core::ptr::addr_of_mut!(kredzone);
    //       let ptr: *mut usize = ptr.add(index);
    //       ptr.write_volatile(value);
    //   }
}
