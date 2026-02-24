pub struct Vmem {
    /// Array of user page mappings.
    mappings: [PageMapping; MAX_USER_PAGES],
    /// Number of valid mappings.
    mapping_count: usize,
}
