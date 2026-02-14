# Exec Consistency Fix: sys_capability

## Summary
- Mismatches fixed: 1
- Missing functions added: 0
- Documented equivalences: 2

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `try_from` | RESTORED inline match | The verus version delegated to `try_from_u32(value)` instead of inlining the match. Restored the original inline `match value { 0 => ..., 1 => ..., ... }` pattern to match the source. The `PARSE_ERROR_MESSAGE` constant is used instead of the `"invalid capability"` string literal; they have the same value (`"invalid capability"`), so the runtime behavior is identical. The `0u32` literal suffixes are required by Verus but are semantically equivalent to bare `0` in Rust. |
| `try_from_u32` | KEPT as verification auxiliary | Not present in original source. Documented as a verification auxiliary that provides a standalone verified conversion usable by proofs and other verified code without going through the trait implementation. This follows the same pattern used in `pid.rs` and `tid.rs`. |
| `to_u32` | KEPT as verification auxiliary | Not present in original source. Documented as a verification auxiliary that enables round-trip proofs (`try_from_u32(cap.to_u32()) == Ok(cap)`). Already had documentation noting it is not in the original source. |

## Verification: PASS
- 14 verified, 0 errors.
- No `assume`, `admit`, or unjustified `external_body` used.
