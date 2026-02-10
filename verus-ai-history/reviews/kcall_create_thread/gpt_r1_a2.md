# Review: kcall_create_thread (gpt-5.2-codex)

## Grade: B-

## Issues Found

### Critical
- None.

### High
- **Location:** `spec_is_error_code_value` + `copy_from_user`/`pm_create_thread` postconditions (spec/exec)
  - **Description:** The error-code domain is now constrained to a small hand-picked subset (2, 3, 12, 14, 16, 22), but `ErrorCode` in the kernel defines many more values. This makes the verification too strong and potentially unsound: if the real code returns any other valid error code, the model rules it out.
  - **Suggested Fix:** Replace `spec_is_error_code_value` with a definition that matches the full `ErrorCode` enum domain (or link to `ErrorCode::get()`/conversion lemmas). If full enumeration is impractical, revert to `code > 0` and add a separate lemma in the PM/VMM modules proving which codes are actually possible for these calls.

### Medium
- **Location:** `CreateThreadInputView.arg0` / `is_user_region` / `is_user_addr` (exec/spec)
  - **Description:** Although `arg0` is now tracked, it is still unused in the validation logic. `args_addr_valid` is an unconstrained boolean; there is no spec-level linkage between `arg0`, the `VirtualAddress::from_raw_value(args.arg0 as usize)` cast, the `size_of::<ThreadCreateArgs>()`, and the boolean passed to `is_user_region`. This leaves the key pointer/size correctness unverified and does not fully establish semantic equivalence.
  - **Suggested Fix:** Carry concrete address/size values in the view and define `spec_args_addr_valid` via a predicate on `arg0` and the size. Alternatively, pass `ghost_arg0` (and size) through `is_user_region`/`copy_from_user` external bodies with linking lemmas that tie the booleans to the concrete values.

### Low
- **Location:** `USER_STACK_SIZE()` constant (spec)
  - **Description:** The minimum stack size is hardcoded as `524288` with only a comment asserting it matches `config::memory_layout::USER_STACK_SIZE`. If the kernel constant changes, the proof may silently drift.
  - **Suggested Fix:** Add an explicit linkage check (e.g., a build-time or CI assertion), or a Verus lemma that proves equality against a shared constant exported for verification.

## Positive Observations
- The stack-size validation is now a concrete numeric comparison instead of a boolean oracle, fixing the previous specification weakness.
- PID is now ghost-tracked and consistently threaded through `copy_from_user` and `pm_create_thread`, improving argument identity tracking.
- Proof structure remains cleanly split and still captures short-circuit behavior and error propagation.

## Summary
Some prior issues are genuinely addressed (stack-size check, PID tracking), but the error-code domain restriction is incorrect and the address-validation linkage is still missing. Until the model ties `arg0` to address checks and relaxes or correctly models the error-code domain, the verification remains incomplete.
