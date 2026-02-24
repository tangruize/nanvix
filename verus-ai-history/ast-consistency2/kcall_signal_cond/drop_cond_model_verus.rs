pub fn drop_cond_model(cond_addr: u32)
    requires
        // A valid Condvar reference must have been acquired via get_cond.
        spec_condvar_acquired(cond_addr as nat),
    ensures
        spec_cond_ref_released(cond_addr as nat),
{
    unimplemented!()
}
