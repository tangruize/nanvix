pub struct ThreadCreateArgsModel {
    /// The user_fn address (concrete value from copied args).
    pub user_fn_addr: u32,
    /// Whether user_fn lies in user address space.
    pub user_fn_valid: bool,
    /// First argument to the user function (passthrough, not validated).
    pub user_fn_arg0: u32,
    /// Second argument to the user function (passthrough, not validated).
    pub user_fn_arg1: u32,
    /// The user_stack_base address (concrete value from copied args).
    pub user_stack_base_addr: u32,
    /// Whether user_stack region lies in user address space.
    pub user_stack_valid: bool,
    /// The user stack size in bytes.
    pub user_stack_size: u32,
    /// Whether user_tda is present.
    pub has_user_tda: bool,
    /// The user_tda address (concrete value from copied args; 0 if absent).
    pub user_tda_addr: u32,
    /// Whether user_tda (if present) lies in user address space.
    pub user_tda_valid: bool,
}
