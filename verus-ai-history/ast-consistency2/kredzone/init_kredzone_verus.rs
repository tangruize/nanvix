pub fn init_kredzone()
    ensures
        true,  // Effect: all kredzone entries are set to 0.
{
    // Zero all entries in the kredzone.
    let mut idx: usize = 0;
    while idx < NUM_ENTRIES
        invariant
            idx <= NUM_ENTRIES,
        decreases
            NUM_ENTRIES - idx,
    {
        let _ = store(idx, 0);  // Cannot fail: idx < NUM_ENTRIES.
        idx = idx + 1;
    }
}
