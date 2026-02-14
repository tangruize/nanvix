# Exec Consistency Fix: process_state

## Summary
- Mismatches fixed: 0
- Missing functions added: 28 (struct + 28 functions moved from submodule to mod.rs)
- Documented equivalences: 0

## Root Cause

The original source defines `ProcessState` and all 28 functions directly in
`src/kernel/src/pm/process/state/mod.rs`. The verus split had moved all
ProcessState code into a separate `process_state.rs` submodule, leaving `mod.rs`
as a thin re-export file. The AST comparison tool compares `mod.rs` to `mod.rs`,
so all 28 functions appeared as MISSING_IN_VERUS.

## Fix

Merged the content of `process_state.rs`, `process_state.spec.rs`, and
`process_state.proof.rs` into `mod.rs`, `mod.spec.rs`, and `mod.proof.rs`
respectively. Removed the `pub mod process_state;` declaration. The old
`process_state.{rs,spec.rs,proof.rs}` files are retained but no longer
imported.

No executable logic was changed — this is a pure structural reorganization
to align the verus file layout with the original source file layout.

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `ProcessState` (struct) | Moved from `process_state.rs` to `mod.rs` | Align with original source layout for AST consistency. |
| `new` | Moved from `process_state.rs` to `mod.rs` | Same as above. |
| `pid` | Moved from `process_state.rs` to `mod.rs` | Same as above. |
| `set_capability` | Moved from `process_state.rs` to `mod.rs` | Same as above. |
| `clear_capability` | Moved from `process_state.rs` to `mod.rs` | Same as above. |
| `has_capability` | Moved from `process_state.rs` to `mod.rs` | Same as above. |
| `vmem` | Covered by `vmem_stub` in `mod.rs` | Opaque HAL boundary type; external_body stub with frame conditions. |
| `vmem_mut` | Covered by `vmem_mut_stub` in `mod.rs` | Opaque HAL boundary type; external_body stub with frame conditions. |
| `copy_from_user_unaligned` | Covered by `copy_from_user_unaligned_stub` in `mod.rs` | Read-only on opaque Vmem; external_body stub. |
| `copy_to_user_unaligned` | Covered by `copy_to_user_unaligned_stub` in `mod.rs` | Read-only on opaque Vmem; external_body stub. |
| `add_event` | Covered by `add_event_stub` in `mod.rs` | Opaque EventOwnership; external_body stub with frame conditions. |
| `remove_event` | Covered by `remove_event_stub` in `mod.rs` | Opaque EventOwnership; external_body stub with frame conditions. |
| `post_message` | Covered by `post_message_stub` in `mod.rs` | Opaque Mailbox; external_body stub with frame conditions. |
| `receive_message` | Covered by `receive_message_stub` in `mod.rs` | Opaque Mailbox; external_body stub with frame conditions. |
| `add_mmio` | Covered by `add_mmio_stub` in `mod.rs` | Opaque IoMemoryRegion; external_body stub with frame conditions. |
| `remove_mmio` | Covered by `remove_mmio_stub` in `mod.rs` | Opaque IoMemoryRegion; external_body stub with frame conditions. |
| `add_pmio` | Moved from `process_state.rs` to `mod.rs` | Fully verified exec function. |
| `remove_pmio` | Moved from `process_state.rs` to `mod.rs` | Fully verified exec function. |
| `get_pmio` | Covered by `get_pmio_stub` in `mod.rs` | Private helper on opaque AnyIoPort; external_body stub. |
| `get_pmio_mut` | Covered by `get_pmio_mut_stub` in `mod.rs` | Private helper on opaque AnyIoPort; external_body stub. |
| `read_pmio` | Covered by `read_pmio_stub` in `mod.rs` | Read-only on opaque AnyIoPort; external_body stub. |
| `write_pmio` | Covered by `write_pmio_stub` in `mod.rs` | Opaque AnyIoPort; external_body stub with frame conditions. |
| `get_mutex` | Moved from `process_state.rs` to `mod.rs` | Fully verified exec function. |
| `put_mutex` | Moved from `process_state.rs` to `mod.rs` | Fully verified exec function. |
| `get_cond` | Moved from `process_state.rs` to `mod.rs` | Fully verified exec function. |
| `put_cond` | Moved from `process_state.rs` to `mod.rs` | Fully verified exec function. |
| `fmt` | Covered by `debug_fmt_stub` in `mod.rs` | Formatting is outside verification scope; external_body stub. |
| `state` | Covered by `ProcessRef::state_stub` in `mod.rs` | Lifecycle accessor on opaque enum; external_body stub. |
| `state_mut` | Covered by `ProcessRefMut::state_mut_stub` in `mod.rs` | Lifecycle accessor on opaque enum; external_body stub. |

## Verification: PASS
- Module `kernel::pm::process::state`: 248 verified, 0 errors
- Module `kernel::pm::process::state::running`: 44 verified, 0 errors
- Module `kernel::pm::process::state::zombie`: 25 verified, 0 errors
- Module `kernel::pm::process::state::interrupted`: 37 verified, 0 errors
