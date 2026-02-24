# Exec Diff: capability

**Source:** `/home/ubuntu/nanvix/src/kernel/src/pm/process/capability.rs`
**Verus:** `/home/ubuntu/nanvix/verus/split/kernel/pm/process/capability.rs`

| Function | Status | Files |
|----------|--------|-------|
| `clear` | MISMATCH | clear_source.rs, clear_verus.rs, clear.diff |
| `has` | MISMATCH | has_source.rs, has_verus.rs, has.diff |
| `set` | MISMATCH | set_source.rs, set_verus.rs, set.diff |
| `default` | EXTRA_IN_VERUS | default_verus.rs (EXTRA) |
| `new` | EXTRA_IN_VERUS | new_verus.rs (EXTRA) |
| `to_mask` | EXTRA_IN_VERUS | to_mask_verus.rs (EXTRA) |

## Struct Issues

| Struct | Status | Files |
|--------|--------|-------|
| `Capabilities` | MISMATCH | struct_Capabilities_source.rs, struct_Capabilities_verus.rs, struct_Capabilities.diff |
