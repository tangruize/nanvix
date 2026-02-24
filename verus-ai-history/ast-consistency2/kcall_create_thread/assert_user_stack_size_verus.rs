pub fn assert_user_stack_size(runtime_size: u32)
    requires
        runtime_size as nat == USER_STACK_SIZE(),
{
}
