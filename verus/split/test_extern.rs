use vstd::prelude::*;

verus! {
    pub extern "C" fn test_verified_extern_c() -> u32 {
        5
    }
}

fn main() {}
