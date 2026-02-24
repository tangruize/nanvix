pub fn create_thread_model(
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
) -> (ret: (KcallResultModel, Ghost<CreateThreadInputView>, Ghost<CreateThreadOutcomeView>))
    requires
        // The minimum stack size parameter matches the spec constant.
        user_stack_size_min as nat == USER_STACK_SIZE(),
        // Copy error code must be valid when copy fails.
        !copy_succeeded ==> spec_is_valid_error_code(copy_error_code as int),
        // The args_size parameter matches the ThreadCreateArgs struct size.
        ghost_args_size == THREAD_CREATE_ARGS_SIZE(),
        // Ghost addresses must match the concrete addresses in thread_args.
        // This ensures validation booleans correspond to the copied args.
        ghost_user_fn_addr == thread_args.user_fn_addr as nat,
        ghost_user_stack_base_addr == thread_args.user_stack_base_addr as nat,
        ghost_user_tda_addr == thread_args.user_tda_addr as nat,
    ensures
        // Build the ghost input from parameters.
        ret.1@ == (CreateThreadInputView {
            pid: ghost_pid,
            arg0: ghost_arg0,
            args_size: ghost_args_size,
            user_fn_addr: ghost_user_fn_addr,
            user_stack_base_addr: ghost_user_stack_base_addr,
            user_tda_addr: ghost_user_tda_addr,
            args_addr_valid: args_addr_valid,
            copy_succeeded: copy_succeeded,
            copy_error_code: copy_error_code as int,
            thread_args: thread_args.spec_view(),
        }),
        // The result matches the spec pipeline.
        ret.0.spec_view() == spec_create_thread_result(ret.1@, ret.2@),
        // Args address error path: InvalidArgument returned immediately.
        !args_addr_valid
            ==> ret.0.spec_view() == (CreateThreadResultView::Error {
                    error_code: ERROR_CODE_INVALID_ARGUMENT()
                }),
        // Copy error path: copy error code propagated.
        args_addr_valid && !copy_succeeded
            ==> ret.0.spec_view() == (CreateThreadResultView::Error {
                    error_code: copy_error_code as int
                }),
        // Success path: all validations passed and PM succeeded.
        spec_is_success(ret.0.spec_view())
            ==> spec_all_validations_passed(ret.1@)
                && spec_pm_create_thread_ok(ret.2@),
        // Result is always Success or Error.
        spec_is_success(ret.0.spec_view()) || spec_is_error(ret.0.spec_view()),
        // Success and Error are mutually exclusive.
        !(spec_is_success(ret.0.spec_view()) && spec_is_error(ret.0.spec_view())),
        // Ghost addresses match thread_args concrete addresses (data-flow linkage).
        ret.1@.user_fn_addr == ret.1@.thread_args.user_fn_addr,
        ret.1@.user_stack_base_addr == ret.1@.thread_args.user_stack_base_addr,
        ret.1@.user_tda_addr == ret.1@.thread_args.user_tda_addr,
        // Args size matches the spec constant.
        ret.1@.args_size == THREAD_CREATE_ARGS_SIZE(),
{
    // Build the ghost input view.
    let ghost input_view: CreateThreadInputView = CreateThreadInputView {
        pid: ghost_pid,
        arg0: ghost_arg0,
        args_size: ghost_args_size,
        user_fn_addr: ghost_user_fn_addr,
        user_stack_base_addr: ghost_user_stack_base_addr,
        user_tda_addr: ghost_user_tda_addr,
        args_addr_valid: args_addr_valid,
        copy_succeeded: copy_succeeded,
        copy_error_code: copy_error_code as int,
        thread_args: thread_args.spec_view(),
    };

    // Step 1: Check if thread_create_args lies in user space.
    let addr_valid: bool = is_user_region(args_addr_valid, Ghost(ghost_arg0), Ghost(ghost_args_size));
    if !addr_valid {
        let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
        proof {
            lemma_args_addr_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: ErrorCode::InvalidArgument as i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 2: Copy thread_create_args from user space.
    // The copy returns the ThreadCreateArgsModel on success, creating an
    // explicit data flow from copy output to validation input (steps 3-5).
    let copy_result: CopyFromUserResultModel = copy_from_user(
        copy_succeeded,
        copy_error_code,
        thread_args,
        Ghost(ghost_pid),
        Ghost(ghost_arg0),
    );
    let copied_args: ThreadCreateArgsModel = match copy_result {
        CopyFromUserResultModel::CopyError { error_code } => {
            let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
            proof {
                lemma_copy_error_propagates(input_view, pm_view);
                lemma_result_exhaustive(input_view, pm_view);
            }
            return (
                KcallResultModel::Error { error_code },
                Ghost(input_view),
                Ghost(pm_view),
            );
        },
        CopyFromUserResultModel::CopyOk { args } => {
            // copy_from_user postcondition guarantees: args.spec_view() == thread_args.spec_view().
            // This links the copy output to input_view.thread_args.
            proof {
                lemma_copy_output_determines_validation(input_view, args.spec_view());
            }
            args
        },
    };

    // Steps 3-6 use `copied_args` — the ThreadCreateArgsModel returned by
    // copy_from_user. This is the structural copy-to-validation linkage:
    // validation operates on the copy output, not an independent parameter.

    // Step 3: Check user_fn lies in user address space.
    let fn_valid: bool = is_user_addr(copied_args.user_fn_valid, Ghost(ghost_user_fn_addr));
    if !fn_valid {
        let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
        proof {
            lemma_user_fn_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: ErrorCode::InvalidArgument as i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 4: Check user_stack lies in user address space.
    let stack_region_valid: bool = is_user_region(
        copied_args.user_stack_valid,
        Ghost(ghost_user_stack_base_addr),
        Ghost(copied_args.user_stack_size as nat),
    );
    if !stack_region_valid {
        let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
        proof {
            lemma_user_stack_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: ErrorCode::InvalidArgument as i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 4b: Check user_stack_size >= USER_STACK_SIZE (concrete numeric comparison).
    if copied_args.user_stack_size < user_stack_size_min {
        let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
        proof {
            lemma_user_stack_invalid_propagates(input_view, pm_view);
            lemma_result_exhaustive(input_view, pm_view);
        }
        return (
            KcallResultModel::Error { error_code: ErrorCode::InvalidArgument as i32 },
            Ghost(input_view),
            Ghost(pm_view),
        );
    }

    // Step 5: Check user_tda (if present) lies in user address space.
    if copied_args.has_user_tda {
        let tda_valid: bool = is_user_addr(copied_args.user_tda_valid, Ghost(ghost_user_tda_addr));
        if !tda_valid {
            let ghost pm_view: CreateThreadOutcomeView = IRRELEVANT_PM_OUTCOME();
            proof {
                lemma_user_tda_invalid_propagates(input_view, pm_view);
                lemma_result_exhaustive(input_view, pm_view);
            }
            return (
                KcallResultModel::Error { error_code: ErrorCode::InvalidArgument as i32 },
                Ghost(input_view),
                Ghost(pm_view),
            );
        }
    } else {
        proof {
            lemma_absent_tda_always_valid(input_view);
        }
    }

    // Step 6: All validations passed. Call PM create_thread with ghost argument identity.
    let pm_result: CreateThreadResultModel =
        pm_create_thread(Ghost(ghost_pid), Ghost(copied_args.spec_view()));
    let ghost pm_view: CreateThreadOutcomeView = pm_result.spec_view();

    proof {
        lemma_result_exhaustive(input_view, pm_view);
    }

    match pm_result {
        CreateThreadResultModel::CtOk { tid } => {
            let tid_value: i32 = tid;
            (
                KcallResultModel::Success { tid_value },
                Ghost(input_view),
                Ghost(pm_view),
            )
        },
        CreateThreadResultModel::CtError { error_code } => {
            (
                KcallResultModel::Error { error_code },
                Ghost(input_view),
                Ghost(pm_view),
            )
        },
    }
}
