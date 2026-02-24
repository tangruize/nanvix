    pub fn new() -> (result: Self)
        ensures
            result.inv(),
            result@.mapping_count == 0,
    {
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
