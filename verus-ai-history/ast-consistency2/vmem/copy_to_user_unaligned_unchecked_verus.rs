    pub fn copy_to_user_unaligned_unchecked(
        &self,
        dst: usize,
        src: usize,
        size: usize,
        dry_run: bool,
    ) -> (result: Result<(), Error>)
        requires
            self.inv(),
            size > 0 ==> self@.spec_user_region_is_mapped(dst as int, size as int),
        ensures
            result.is_ok() ==> {
                &&& size > 0
                &&& spec_is_kernel_region(src as int, size as int)
                &&& spec_is_user_region(dst as int, size as int)
                &&& spec_is_physical_region(src as int, size as int)
            },
    {
        unimplemented!()
    }
