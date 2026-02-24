pub struct ZombieHarvestResult {
    /// Whether a zombie was found.
    pub found: bool,
    /// Whether harvesting failed with an error.
    pub error: bool,
    /// Process identifier of the harvested zombie.
    pub pid: u32,
    /// Whether the harvested zombie was the init daemon.
    pub is_initd: bool,
    /// Exit status of the harvested zombie.
    pub exit_status: u32,
}
