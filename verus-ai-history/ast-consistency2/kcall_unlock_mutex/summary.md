# Exec Diff: unlock_mutex

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/kcall/unlock_mutex.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/kcall/unlock_mutex.rs`

| Function | Status | Files |
|----------|--------|-------|
| `unlock_mutex` | MISMATCH | unlock_mutex_source.rs, unlock_mutex_verus.rs, unlock_mutex.diff |
| `drop_guard_model` | EXTRA_IN_VERUS | drop_guard_model_verus.rs (EXTRA) |
| `take_mutex_guard_model` | EXTRA_IN_VERUS | take_mutex_guard_model_verus.rs (EXTRA) |
| `unlock_mutex_model` | EXTRA_IN_VERUS | unlock_mutex_model_verus.rs (EXTRA) |
