# Exec Consistency Fix: sys_capability

## Summary
- Mismatches fixed: 1 (libs) + 1 (kernel)
- Missing functions added: 0
- Documented equivalences: 4

## Modules
Both the libs and kernel copies of `capability.rs` were fixed and verified:
- **Libs:** `verus/split/libs/sys/sys/pm/capability.rs` → `libs::sys::sys::pm::capability`
- **Kernel:** `verus/split/kernel/pm/sys/capability.rs` → `kernel::pm::sys::capability`

## Changes
| Function / Item | Action | Justification |
|-----------------|--------|---------------|
| `try_from` (libs + kernel) | RESTORED inline match | The verus versions delegated to `try_from_u32(value)` instead of inlining the match. Restored the original inline `match value { 0 => ..., 1 => ..., ... }` pattern to match the source. The `0u32` literal suffixes are required by Verus but are semantically equivalent to bare `0` in Rust. |
| `try_from_u32` (libs + kernel) | KEPT as verification auxiliary | Not present in original source. Documented as a verification auxiliary that provides a standalone verified conversion usable by proofs and other verified code without going through the trait implementation. Follows the same pattern used in `pid.rs` and `tid.rs`. |
| `to_u32` (libs + kernel) | KEPT as verification auxiliary | Not present in original source. Documented as a verification auxiliary that enables round-trip proofs (`try_from_u32(cap.to_u32()) == Ok(cap)`). |
| `PARSE_ERROR_MESSAGE` constant | Documented structural addition | Not present in original source (which uses `"invalid capability"` literal directly). Introduced as `pub const PARSE_ERROR_MESSAGE: &'static str = "invalid capability"` in both libs and kernel copies. Runtime-equivalent; the constant has the identical value. Added for DRY and to enable postcondition references to the error message. |
| `PartialEq, Eq` derives | Documented structural addition | The original source derives `Debug, Clone, Copy`. The verus versions add `PartialEq, Eq`. These are required by Verus for equality reasoning on enum values and do not alter runtime behavior (they add trait impls that the original lacked but do not change existing behavior). |

## Verification: PASS
- **Libs** (`libs::sys::sys::pm::capability`): 14 verified, 0 errors.
- **Kernel** (`kernel::pm::sys::capability`): 14 verified, 0 errors.
- No `assume`, `admit`, or unjustified `external_body` used in either copy.
