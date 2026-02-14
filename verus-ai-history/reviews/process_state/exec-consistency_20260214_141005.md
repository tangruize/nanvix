# Review: process_state Exec Consistency (claude-opus-4.6)

## Grade: A-

## Issues Found

### Critical

- None.

### Major

- **`new()` signature diverges from original**: The original `ProcessState::new(pid, vmem)` takes two parameters (ProcessIdentifier and Vmem), while the Verus version takes only `pid`. This is acceptable because `Vmem` is an opaque HAL boundary type excluded from the verification model, but the divergence is not documented in the consistency report's function table — the entry for `new` says "Moved from `process_state.rs` to `mod.rs`" without noting the signature change. A documentation annotation in the Verus code or the fix report should explicitly note this parameter elision.

- **`get_mutex` / `put_mutex` / `get_cond` / `put_cond` signature changes**: The original functions take a single address parameter (e.g., `MutexAddress`) and perform lookup internally. The Verus versions require pre-computed lookup results as extra parameters (`already_present`, `idx`, `ref_count_at_threshold`). This is a standard verification technique (hoisting nondeterministic lookups to preconditions), but these functions are listed in the fix report as simply "Moved" or "Fully verified exec function" without documenting the signature changes. This should be noted as a deliberate verification abstraction.

- **`remove_pmio` return type change**: The original returns `Result<AnyIoPort, Error>` (returning the removed port), while the Verus version returns `Result<(), Error>`. This loses information about the returned port — acceptable since `AnyIoPort` is opaque, but undocumented.

- **`add_pmio` parameter type change**: The original takes `AnyIoPort` (the full port object), while the Verus version takes `u16` (just the port number). Consistent with the PMIO abstraction model but undocumented in the fix report.

### Minor

- **`ProcessState` struct fields are `pub`**: The original uses private fields with getter methods. The Verus version marks all fields `pub` with a documented justification (Verus View trait requirement). This is correctly documented in a field-level comment.

- **`ProcessRefMut` / `ProcessRef` lifetime elision**: The original uses `ProcessRefMut<'a>` / `ProcessRef<'a>` with lifetime parameters and enum variants referencing specific process states. The Verus versions are opaque `external_body` structs with `_phantom: ()`. Acceptable for lifecycle accessors outside verification scope.

- **Old `process_state.{rs,spec.rs,proof.rs}` files retained**: The fix report states these are "retained but no longer imported." Confirmed — no `mod process_state` declaration exists. These dead files should ideally be deleted or moved to an archive to avoid confusion about which files are active.

- **Consistency report inaccuracy**: The report claims "28 functions moved" but several functions were not moved verbatim — they were re-implemented with different signatures or replaced with stubs. The table does distinguish "Moved" vs "Covered by ... stub" but the summary line "28 functions moved" is misleading.

## Verification Results

- `kernel::pm::process::state`: **248 verified, 0 errors** ✅
- Submodules (interrupted, runnable, running, sleeping, zombie): verified as part of the same run.
- The old `process_state` module is no longer registered in the Verus module tree (confirmed: `--verify-module kernel::pm::process::state::process_state` fails with "module not found").

## Function-by-Function Coverage Assessment

| Original Function | Verus Coverage | Faithful? |
|---|---|---|
| `new(pid, vmem)` | `new(pid)` — vmem elided | ✅ (model justified) |
| `pid()` | `pid()` — identical | ✅ |
| `set_capability()` | `set_capability()` — identical | ✅ |
| `clear_capability()` | `clear_capability()` — identical | ✅ |
| `has_capability()` | `has_capability()` — identical | ✅ |
| `vmem()` | `vmem_stub()` — external_body | ✅ (opaque) |
| `vmem_mut()` | `vmem_mut_stub()` — external_body, frame conditions | ✅ (opaque) |
| `copy_from_user_unaligned()` | `copy_from_user_unaligned_stub()` — external_body, &self | ✅ (opaque) |
| `copy_to_user_unaligned()` | `copy_to_user_unaligned_stub()` — external_body, &self | ✅ (opaque) |
| `add_event()` | `add_event_stub()` — external_body, frame conditions | ✅ (opaque) |
| `remove_event()` | `remove_event_stub()` — external_body, frame conditions | ✅ (opaque) |
| `post_message()` | `post_message_stub()` — external_body, frame conditions | ✅ (opaque) |
| `receive_message()` | `receive_message_stub()` — external_body, frame conditions | ✅ (opaque) |
| `add_mmio()` | `add_mmio_stub()` — external_body, frame conditions | ✅ (opaque) |
| `remove_mmio()` | `remove_mmio_stub()` — external_body, frame conditions | ✅ (opaque) |
| `add_pmio(AnyIoPort)` | `add_pmio(u16)` — fully verified | ✅ (abstracted) |
| `remove_pmio(u16) → Result<AnyIoPort, Error>` | `remove_pmio(u16, bool, usize) → Result<(), Error>` — fully verified | ✅ (abstracted) |
| `get_pmio()` | `get_pmio_stub()` — external_body, &self | ✅ (opaque) |
| `get_pmio_mut()` | `get_pmio_mut_stub()` — external_body, frame conditions | ✅ (opaque) |
| `read_pmio()` | `read_pmio_stub()` — external_body, &self | ✅ (opaque) |
| `write_pmio()` | `write_pmio_stub()` — external_body, frame conditions | ✅ (opaque) |
| `get_mutex(MutexAddress)` | `get_mutex(u64, bool, usize)` — fully verified | ✅ (abstracted) |
| `put_mutex(MutexAddress)` | `put_mutex(u64, bool, bool, usize)` — fully verified | ✅ (abstracted) |
| `get_cond(ConditionAddress)` | `get_cond(u64, bool, usize)` — fully verified | ✅ (abstracted) |
| `put_cond(ConditionAddress)` | `put_cond(u64, bool, bool, usize)` — fully verified | ✅ (abstracted) |
| `fmt (Debug)` | `debug_fmt_stub()` — external_body | ✅ (formatting) |
| `state()` | `ProcessRef::state_stub()` — external_body | ✅ (lifecycle) |
| `state_mut()` | `ProcessRefMut::state_mut_stub()` — external_body | ✅ (lifecycle) |

## Equivalence Justification Assessment

| Category | Sound? | Notes |
|---|---|---|
| Opaque HAL stubs (vmem, events, mailbox, mmio, pmio r/w) | ✅ | Frame conditions correctly assert verified state is preserved. Read-only stubs (`&self`) trivially safe. |
| Mutex/condvar parallel Vec model | ✅ | BTreeMap abstracted to parallel Vec pairs with uniqueness invariant. Capacity bounds match `kernel_config.toml` (32/32). Ref-count thresholds match original `extract_if` predicates (≤2 for mutex, ≤1 for condvar). |
| PMIO Vec model | ✅ | LinkedList abstracted to Vec with first-occurrence removal semantics matching `iter().position()` + `remove()`. |
| ProcessRef/ProcessRefMut opaque wrappers | ✅ | Lifecycle dispatch logic is outside verification scope; stubs have minimal postconditions. |

## Summary

The consistency fix successfully restructured the Verus code from a `process_state.rs` submodule into `mod.rs`, aligning the file layout with the original source for AST tool compatibility. All 28 original functions are accounted for: 10 are fully verified exec implementations (new, pid, set/clear/has_capability, get/put_mutex, get/put_cond, add/remove_pmio), and 18 are covered by appropriately justified `external_body` stubs with frame conditions. Verification passes cleanly (248/0).

The primary weakness is **documentation fidelity**: the fix report understates the scope of changes by characterizing signature-changed functions as "Moved" when they were actually re-implemented with abstracted parameters. The `new()` function's dropped `vmem` parameter and the mutex/condvar functions' hoisted lookup parameters should be explicitly documented as verification abstractions. The old dead `process_state.{rs,spec.rs,proof.rs}` files should be cleaned up.

The verification model itself is sound: the protocol-level properties (capacity enforcement, ref-count cleanup, PID immutability, error codes) are correctly proven, and frame conditions on opaque boundaries are well-structured.
