# Exec Diff: create_thread

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/create_thread.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/create_thread.rs`

| Function | Status | Files |
|----------|--------|-------|
| `create_thread` | MISMATCH | create_thread_source.rs, create_thread_verus.rs, create_thread.diff |
| `assert_user_stack_size` | EXTRA_IN_VERUS | assert_user_stack_size_verus.rs (EXTRA) |
| `copy_from_user` | EXTRA_IN_VERUS | copy_from_user_verus.rs (EXTRA) |
| `create_thread_model` | EXTRA_IN_VERUS | create_thread_model_verus.rs (EXTRA) |
| `is_user_addr` | EXTRA_IN_VERUS | is_user_addr_verus.rs (EXTRA) |
| `is_user_region` | EXTRA_IN_VERUS | is_user_region_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `ThreadCreateArgsModel` | EXTRA_IN_VERUS | struct_ThreadCreateArgsModel_verus.rs (EXTRA) |
