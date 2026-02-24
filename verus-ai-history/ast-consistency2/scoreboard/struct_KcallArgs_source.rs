pub struct KcallArgs {
    pub pid: ProcessIdentifier,
    pub tid: ThreadIdentifier,
    pub number: u32,
    pub arg0: u32,
    pub arg1: u32,
    pub arg2: u32,
    pub arg3: u32,
}
