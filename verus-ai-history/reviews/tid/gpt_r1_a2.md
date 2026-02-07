# Review: tid (gpt-5.2-codex)

## Grade: B

## Issues Found

### Critical
- None.

### High
- None.

### Medium
- **Priority:** Medium  
  **Location:** `ThreadIdentifier` struct field (`verus/split/kernel/pm/sys/tid.rs:72-75`)  
  **Description:** The `value` field is still `pub`, so the API surface remains wider than the original tuple struct (`src/libs/sys/src/sys/pm/tid.rs:41`). The comment claims it is for Verus spec access, but the underlying API mismatch is unchanged and still allows direct construction/mutation outside the intended conversion functions.  
  **Suggested Fix:** Make the field private and keep spec access via `view()`/`spec_value` inside the module (or gate any special access behind `pub(crate)` helpers used only in proofs).

- **Priority:** Medium  
  **Location:** Extra public inherent methods (`verus/split/kernel/pm/sys/tid.rs:105-562`, esp. `from_i32/into_i32/into_isize/into_i64`, `try_from_*`, `try_into_*`, and `eq/ne/lt/le/gt/ge/cmp_ord`).  
  **Description:** The verified exec module still exposes a large set of public inherent methods that do not exist in the original type (which only provides `to_ne_bytes`/`from_ne_bytes` and trait impls). This expands the public API beyond the source, so the interface is not equivalent even if the behavior matches.  
  **Suggested Fix:** Make these methods `pub(crate)` or move them into a private proof-only module, and rely on the trait impls for the public API to match the original surface.

### Low
- None.

## Positive Observations
- The previous `Default` implementation is no longer present in the verified exec module.
- All conversions still preserve error codes/messages (`InvalidArgument`, "invalid thread identifier").
- No new `unimplemented!`/`assume`/`admit` sites were introduced in the updated tid modules.

## Summary
The re-review confirms one prior issue was fixed (the extra `Default` impl), but the two API surface mismatches remain: the `value` field is still public and the verified exec module still exports extra inherent methods not present in the original. These are not verification unsoundness issues, but they keep the verified interface from matching the production API, so the review cannot pass yet. Verification appears structurally complete (no missing proofs), but equivalence is incomplete until the API is aligned.
