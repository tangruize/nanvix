    pub fn base(&self) -> (result: PageAddress)
        requires
            self.inv(),
        ensures
            result.inv(),
            result@.raw_value() == self@.page_address(),
            result@.is_aligned(),
    {
        proof {
            // Use lemma to connect closed specs to FrameAddress properties.
            self.kframe.lemma_alignment_connection();
        }
        // Original code: PageAddress::new(self.kframe.base().into_page_address().into_virtual_address())
        // Verus limitation: into_page_address()/into_virtual_address() are not available on the
        // simplified verus FrameAddress. Under identity mapping, both chains produce the same raw
        // address value (see module-level docs for equivalence proof).
        let frame_addr: FrameAddress = self.kframe.base();
        proof {
            frame_addr.lemma_inv_from_aligned();
        }
        PageAddress::new(frame_addr.into_raw_value())
    }
