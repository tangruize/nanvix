    pub fn clone(from: &Self) -> (result: Self)
        requires
            from.inv(),
        ensures
            result.inv(),
            result@.mapping_count == 0,
            // Note: Source preservation is guaranteed by Rust's borrow checker.
            // The `from: &Self` parameter is an immutable borrow, so Rust ensures
            // the source is unchanged. Verus's `old()` requires `&mut` so we cannot
            // express this directly in the postcondition, but the type system
            // provides the guarantee.
    {
        // Use from in a proof block to document that we require source validity.
        // Kernel mappings sharing is abstracted - we only verify user mapping properties.
        proof {
            assert(from.inv());
        }

        let empty_mapping: PageMapping = PageMapping {
            vaddr: 0,
            frame_addr: 0,
            valid: false,
        };

        Vmem {
            mappings: [empty_mapping; MAX_USER_PAGES],
            mapping_count: 0,
        }
    }
