    pub fn new(kframe: KernelFrame) -> (result: KernelPage)
        requires
            kframe.spec_is_aligned(),
        ensures
            result.inv(),
            result@.frame_address() == kframe.spec_raw_address(),
            result@.page_address() == kframe.spec_raw_address(),
            result@.pool_id() == kframe.spec_pool_id(),
            result@.is_identity_mapped(),
    {
        proof {
            // Use lemma to connect closed specs to FrameAddress properties.
            kframe.lemma_alignment_connection();
        }
        KernelPage { kframe }
    }
