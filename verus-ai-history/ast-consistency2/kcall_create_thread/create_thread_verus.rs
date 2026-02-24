pub fn create_thread(
    args_addr_valid: bool,
    copy_succeeded: bool,
    copy_error_code: i32,
    thread_args: &ThreadCreateArgsModel,
    user_stack_size_min: u32,
    Ghost(ghost_pid): Ghost<nat>,
    Ghost(ghost_arg0): Ghost<nat>,
    Ghost(ghost_args_size): Ghost<nat>,
    Ghost(ghost_user_fn_addr): Ghost<nat>,
    Ghost(ghost_user_stack_base_addr): Ghost<nat>,
    Ghost(ghost_user_tda_addr): Ghost<nat>,
) -> (result: KcallResultModel)
    requires
        user_stack_size_min as nat == USER_STACK_SIZE(),
        !copy_succeeded ==> spec_is_valid_error_code(copy_error_code as int),
        ghost_args_size == THREAD_CREATE_ARGS_SIZE(),
        ghost_user_fn_addr == thread_args.user_fn_addr as nat,
        ghost_user_stack_base_addr == thread_args.user_stack_base_addr as nat,
        ghost_user_tda_addr == thread_args.user_tda_addr as nat,
    ensures
        spec_is_success(result.spec_view()) || spec_is_error(result.spec_view()),
        !args_addr_valid
            ==> result.spec_view() == (CreateThreadResultView::Error {
                    error_code: ERROR_CODE_INVALID_ARGUMENT()
                }),
        args_addr_valid && !copy_succeeded
            ==> result.spec_view() == (CreateThreadResultView::Error {
                    error_code: copy_error_code as int
                }),
{
    // Delegate to the fully verified model, discarding ghost witnesses.
    let ret: (KcallResultModel, Ghost<CreateThreadInputView>, Ghost<CreateThreadOutcomeView>) =
        create_thread_model(
            args_addr_valid,
            copy_succeeded,
            copy_error_code,
            thread_args,
            user_stack_size_min,
            Ghost(ghost_pid),
            Ghost(ghost_arg0),
            Ghost(ghost_args_size),
            Ghost(ghost_user_fn_addr),
            Ghost(ghost_user_stack_base_addr),
            Ghost(ghost_user_tda_addr),
        );
    ret.0
}
