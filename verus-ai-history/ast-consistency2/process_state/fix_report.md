# Exec Consistency Fix: process_state

## Summary
- Mismatches fixed: 0
- Missing functions added: 28 (struct + 28 functions restructured from submodule to mod.rs)
- Documented equivalences: 8 (signature divergences documented)

## Root Cause

The original source defines `ProcessState` and all 28 functions directly in
`src/kernel/src/pm/process/state/mod.rs`. The verus split had moved all
ProcessState code into a separate `process_state.rs` submodule, leaving `mod.rs`
as a thin re-export file. The AST comparison tool compares `mod.rs` to `mod.rs`,
so all 28 functions appeared as MISSING_IN_VERUS.

## Fix

Merged the content of `process_state.rs`, `process_state.spec.rs`, and
`process_state.proof.rs` into `mod.rs`, `mod.spec.rs`, and `mod.proof.rs`
respectively. Removed the `pub mod process_state;` declaration and deleted
the old `process_state.{rs,spec.rs,proof.rs}` files.

No executable logic was changed — this is a pure structural reorganization
to align the verus file layout with the original source file layout.

## Signature Divergences

The verification model necessarily abstracts several original types and
parameters. These are **deliberate verification abstractions**, not logic
changes:

| Function | Original Signature | Verus Signature | Reason |
|----------|-------------------|-----------------|--------|
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | `new(pid, vmem)` | `new(pid)` | `Vmem` is an opaque HAL boundary type, elided from the model. |
| `get_mutex` [get_mutex.diff](get_mutex.diff) [get_mutex_source.rs](get_mutex_source.rs) [get_mutex_verus.rs](get_mutex_verus.rs) | `get_mutex(MutexAddress)` | `get_mutex(u64, bool, usize)` | `BTreeMap` abstracted to parallel Vecs; lookup result/index hoisted to preconditions. Return type `Mutex` → `u64` (ref count). |
| `put_mutex` [put_mutex.diff](put_mutex.diff) [put_mutex_source.rs](put_mutex_source.rs) [put_mutex_verus.rs](put_mutex_verus.rs) | `put_mutex(MutexAddress)` | `put_mutex(u64, bool, bool, usize)` | Same: `contains_key()` + `extract_if()` results hoisted to preconditions. |
| `get_cond` [get_cond.diff](get_cond.diff) [get_cond_source.rs](get_cond_source.rs) [get_cond_verus.rs](get_cond_verus.rs) | `get_cond(ConditionAddress)` | `get_cond(u64, bool, usize)` | Same pattern as `get_mutex`. Return type `Condvar` → `u64`. |
| `put_cond` [put_cond.diff](put_cond.diff) [put_cond_source.rs](put_cond_source.rs) [put_cond_verus.rs](put_cond_verus.rs) | `put_cond(ConditionAddress)` | `put_cond(u64, bool, bool, usize)` | Same pattern as `put_mutex`. |
| `add_pmio` [add_pmio.diff](add_pmio.diff) [add_pmio_source.rs](add_pmio_source.rs) [add_pmio_verus.rs](add_pmio_verus.rs) | `add_pmio(AnyIoPort)` | `add_pmio(u16)` | `AnyIoPort` is opaque HAL type, abstracted to port number. |
| `remove_pmio` [remove_pmio.diff](remove_pmio.diff) [remove_pmio_source.rs](remove_pmio_source.rs) [remove_pmio_verus.rs](remove_pmio_verus.rs) | `remove_pmio(u16) → Result<AnyIoPort, Error>` | `remove_pmio(u16, bool, usize) → Result<(), Error>` | Position search hoisted; `AnyIoPort` return elided (opaque). |
| `ProcessState` struct | Private fields | `pub` fields | Required by Verus `View` trait `open spec fn view()`. Documented in field-level comments. |

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `ProcessState` (struct) | Restructured from `process_state.rs` to `mod.rs` | Align with original source layout. Fields made `pub` for Verus View trait. |
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | Restructured; vmem param elided | `Vmem` is opaque HAL type outside verification scope. |
| `pid` [pid.diff](pid.diff) [pid_source.rs](pid_source.rs) [pid_verus.rs](pid_verus.rs) | Restructured; identical signature | Direct move, no changes. |
| `set_capability` [set_capability.diff](set_capability.diff) [set_capability_source.rs](set_capability_source.rs) [set_capability_verus.rs](set_capability_verus.rs) | Restructured; identical signature | Direct move, no changes. |
| `clear_capability` [clear_capability.diff](clear_capability.diff) [clear_capability_source.rs](clear_capability_source.rs) [clear_capability_verus.rs](clear_capability_verus.rs) | Restructured; identical signature | Direct move, no changes. |
| `has_capability` [has_capability.diff](has_capability.diff) [has_capability_source.rs](has_capability_source.rs) [has_capability_verus.rs](has_capability_verus.rs) | Restructured; identical signature | Direct move, no changes. |
| `vmem` [vmem_source.rs](vmem_source.rs) | Covered by `vmem_stub` | Opaque HAL boundary type; external_body stub with frame conditions. |
| `vmem_mut` [vmem_mut_source.rs](vmem_mut_source.rs) | Covered by `vmem_mut_stub` | Opaque HAL boundary type; external_body stub with frame conditions. |
| `copy_from_user_unaligned` [copy_from_user_unaligned_source.rs](copy_from_user_unaligned_source.rs) | Covered by `copy_from_user_unaligned_stub` | Read-only on opaque Vmem; external_body stub. |
| `copy_to_user_unaligned` [copy_to_user_unaligned_source.rs](copy_to_user_unaligned_source.rs) | Covered by `copy_to_user_unaligned_stub` | Read-only on opaque Vmem; external_body stub. |
| `add_event` [add_event_source.rs](add_event_source.rs) | Covered by `add_event_stub` | Opaque EventOwnership; external_body stub with frame conditions. |
| `remove_event` [remove_event_source.rs](remove_event_source.rs) | Covered by `remove_event_stub` | Opaque EventOwnership; external_body stub with frame conditions. |
| `post_message` [post_message_source.rs](post_message_source.rs) | Covered by `post_message_stub` | Opaque Mailbox; external_body stub with frame conditions. |
| `receive_message` [receive_message_source.rs](receive_message_source.rs) | Covered by `receive_message_stub` | Opaque Mailbox; external_body stub with frame conditions. |
| `add_mmio` [add_mmio_source.rs](add_mmio_source.rs) | Covered by `add_mmio_stub` | Opaque IoMemoryRegion; external_body stub with frame conditions. |
| `remove_mmio` [remove_mmio_source.rs](remove_mmio_source.rs) | Covered by `remove_mmio_stub` | Opaque IoMemoryRegion; external_body stub with frame conditions. |
| `add_pmio` [add_pmio.diff](add_pmio.diff) [add_pmio_source.rs](add_pmio_source.rs) [add_pmio_verus.rs](add_pmio_verus.rs) | Restructured; param type `AnyIoPort` → `u16` | Opaque port abstracted to port number in verification model. |
| `remove_pmio` [remove_pmio.diff](remove_pmio.diff) [remove_pmio_source.rs](remove_pmio_source.rs) [remove_pmio_verus.rs](remove_pmio_verus.rs) | Restructured; signature abstracted | Position search hoisted to preconditions; return type `AnyIoPort` → `()`. |
| `get_pmio` [get_pmio_source.rs](get_pmio_source.rs) | Covered by `get_pmio_stub` | Private helper on opaque AnyIoPort; external_body stub. |
| `get_pmio_mut` [get_pmio_mut_source.rs](get_pmio_mut_source.rs) | Covered by `get_pmio_mut_stub` | Private helper on opaque AnyIoPort; external_body stub. |
| `read_pmio` [read_pmio_source.rs](read_pmio_source.rs) | Covered by `read_pmio_stub` | Read-only on opaque AnyIoPort; external_body stub. |
| `write_pmio` [write_pmio_source.rs](write_pmio_source.rs) | Covered by `write_pmio_stub` | Opaque AnyIoPort; external_body stub with frame conditions. |
| `get_mutex` [get_mutex.diff](get_mutex.diff) [get_mutex_source.rs](get_mutex_source.rs) [get_mutex_verus.rs](get_mutex_verus.rs) | Restructured; signature abstracted | BTreeMap lookup hoisted to preconditions; return type `Mutex` → `u64`. |
| `put_mutex` [put_mutex.diff](put_mutex.diff) [put_mutex_source.rs](put_mutex_source.rs) [put_mutex_verus.rs](put_mutex_verus.rs) | Restructured; signature abstracted | BTreeMap lookup + extract_if hoisted to preconditions. |
| `get_cond` [get_cond.diff](get_cond.diff) [get_cond_source.rs](get_cond_source.rs) [get_cond_verus.rs](get_cond_verus.rs) | Restructured; signature abstracted | Same abstraction pattern as get_mutex. |
| `put_cond` [put_cond.diff](put_cond.diff) [put_cond_source.rs](put_cond_source.rs) [put_cond_verus.rs](put_cond_verus.rs) | Restructured; signature abstracted | Same abstraction pattern as put_mutex. |
| `fmt` [fmt_source.rs](fmt_source.rs) | Covered by `debug_fmt_stub` | Formatting is outside verification scope; external_body stub. |
| `state` [state_source.rs](state_source.rs) | Covered by `ProcessRef::state_stub` | Lifecycle accessor on opaque enum; external_body stub. |
| `state_mut` [state_mut_source.rs](state_mut_source.rs) | Covered by `ProcessRefMut::state_mut_stub` | Lifecycle accessor on opaque enum; external_body stub. |

## Cleanup
- Deleted dead files: `process_state.rs`, `process_state.spec.rs`, `process_state.proof.rs`.

## Verification: PASS
- Module `kernel::pm::process::state`: 248 verified, 0 errors
- Module `kernel::pm::process::state::running`: 44 verified, 0 errors
- Module `kernel::pm::process::state::zombie`: 25 verified, 0 errors
- Module `kernel::pm::process::state::interrupted`: 37 verified, 0 errors
