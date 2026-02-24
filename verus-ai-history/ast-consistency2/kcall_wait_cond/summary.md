# Exec Diff: wait_cond

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/wait_cond.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/wait_cond.rs`

| Function | Status | Files |
|----------|--------|-------|
| `wait_cond` | MISMATCH | wait_cond_source.rs, wait_cond_verus.rs, wait_cond.diff |
| `cond_wait_model` | EXTRA_IN_VERUS | cond_wait_model_verus.rs (EXTRA) |
| `get_cond_and_wait_model` | EXTRA_IN_VERUS | get_cond_and_wait_model_verus.rs (EXTRA) |
| `mutex_lock_model` | EXTRA_IN_VERUS | mutex_lock_model_verus.rs (EXTRA) |
| `parse_timeout_model` | EXTRA_IN_VERUS | parse_timeout_model_verus.rs (EXTRA) |
| `wait_cond_model` | EXTRA_IN_VERUS | wait_cond_model_verus.rs (EXTRA) |
