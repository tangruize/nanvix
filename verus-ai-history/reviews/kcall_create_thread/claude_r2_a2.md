# Review: kcall_create_thread (claude-opus-4.6)

## Grade: A

## Previous Issues Disposition

### HIGH: Ghost address parameters for is_user_region/is_user_addr — PARTIALLY FIXED

**Claim:** Ghost address parameters added to `is_user_region` and `is_user_addr`.

**Verification:** Confirmed. The external bodies now accept ghost parameters:
- `is_user_region(valid, Ghost(ghost_addr), Ghost(ghost_size))` (exec line 324–328)
- `is_user_addr(valid, Ghost(ghost_addr))` (exec line 343–346)

The `CreateThreadInputView` now carries `args_size`, `user_fn_addr`, `user_stack_base_addr`, `user_tda_addr` (spec lines 84–90), and the exec threads them through every validation call:
- Step 1: `is_user_region(args_addr_valid, Ghost(ghost_arg0), Ghost(ghost_args_size))` (exec line 522)
- Step 3: `is_user_addr(thread_args.user_fn_valid, Ghost(ghost_user_fn_addr))` (exec line 557)
- Step 4: `is_user_region(thread_args.user_stack_valid, Ghost(ghost_user_stack_base_addr), Ghost(thread_args.user_stack_size as nat))` (exec line 572–576)
- Step 5: `is_user_addr(thread_args.user_tda_valid, Ghost(ghost_user_tda_addr))` (exec line 606)

**Residual gap:** The postconditions on `is_user_region` and `is_user_addr` still only guarantee `result == valid` (exec lines 330, 348). The ghost address/size parameters are **recorded but not constrained** — there is no postcondition like `valid == spec_vmem_is_user_region(ghost_addr, ghost_size)`. This means within this module, the ghost addresses are traceability annotations rather than verification constraints. A caller cannot derive from the postcondition alone that `valid` actually reflects the result of validating the given address.

This is a genuine improvement: the ghost parameters enable cross-module linking lemmas (a VMM module proof could now relate `ghost_addr`/`ghost_size` to the concrete validation result). The documentation (exec lines 117–147) honestly describes this. But the core boolean abstraction gap is **narrowed, not closed**. Downgraded from High to Medium.

### MEDIUM: u32 vs usize architecture dependency — ADDRESSED (Documentation)

Documentation added at exec lines 225–227 explicitly noting the x86-32 dependency and the need for update on 64-bit targets. Reasonable resolution for a documentation-level issue.

### MEDIUM: user_fn_arg0/user_fn_arg1 omitted — ADDRESSED (Documentation)

Documentation added at exec lines 216–221 explaining that these fields are not validated by `create_thread` and passthrough verification is out of scope. This is correct — the original source confirms these fields are never inspected, only forwarded to `pm.create_thread`.

### MEDIUM: spec_is_valid_error_code overly permissive — NOT ADDRESSED

`spec_is_valid_error_code` still just checks `code > 0` (spec line 248–249). The `copy_from_user` postcondition (exec line 376) still uses `spec_is_valid_error_code` rather than `spec_is_error_code_value`. However, the existing documentation (spec lines 252–266) already thoroughly explains this design choice, and the `lemma_error_code_value_implies_valid` proof (proof lines 367–373) provides the strengthening bridge when needed. This is an acceptable intentional trade-off for a per-module verification approach. Downgraded to Low.

### LOW: Hardcoded 22i32 — FIXED

All error return paths now use `ErrorCode::InvalidArgument as i32` (exec lines 530, 565, 584, 598, 614). Verified against original.

### LOW: Dummy PM outcome — FIXED

`IRRELEVANT_PM_OUTCOME()` spec function defined (spec lines 283–285) returning `CreateThreadOutcomeView::CtError { error_code: 0 }`. Used on all early-return paths (exec lines 524, 540, 559, 578, 592, 608). Intent is now self-documenting.

### LOW: Ghost address fields missing from CreateThreadInputView — FIXED

`CreateThreadInputView` now includes `args_size`, `user_fn_addr`, `user_stack_base_addr`, `user_tda_addr` (spec lines 84–90). These are threaded through exec and exposed in the postcondition (exec lines 474–485).

## Issues Found

### Critical

None.

### High

None.

### Medium

- **Location:** `is_user_region` / `is_user_addr` postconditions (exec: create_thread.rs:329–330, 347–348)
  **Description:** Ghost address parameters were added (addressing the previous review's primary concern), but the postconditions do not constrain the relationship between `valid` and the ghost addresses. The postcondition is still just `result == valid`. This means the ghost addresses enable cross-module composition but do not provide intra-module enforcement. An incorrect boolean passed alongside the correct ghost address would not be caught by this module's verification alone. The documentation (exec lines 132–137) correctly acknowledges this.
  **Suggested Fix:** This is an inherent limitation of the per-module approach. If end-to-end linking is ever pursued, add postconditions like `result == spec_vmem_is_user_region(ghost_addr, ghost_size)` by importing the VMM spec. No action needed for this module in isolation.

- **Location:** `spec_is_valid_error_code` remains `code > 0` (spec: create_thread.spec.rs:248–249)
  **Description:** As noted previously, this predicate is overly permissive. The `copy_from_user` external body postcondition guarantees only `code > 0` rather than constraining to actual `ErrorCode` discriminants. This is a deliberate design choice with documented rationale. The `lemma_error_code_value_implies_valid` bridge lemma (proof lines 367–373) enables strengthening at call sites. Acceptable for the current per-module scope.
  **Suggested Fix:** No immediate action required. If copy_from_user's error codes are precisely known, a future revision could strengthen the postcondition.

### Low

- **Location:** `user_stack_size` ghost address threading (exec: create_thread.rs:575)
  **Description:** At Step 4, the stack size passed to `is_user_region` is `Ghost(thread_args.user_stack_size as nat)`. In the original code, `is_user_region` receives `thread_create_args.user_stack_size` which is a `usize` field from the copied struct. The model passes `thread_args.user_stack_size` (a `u32`) cast to `nat`. While this is correct for x86-32, note that the `CreateThreadInputView` does NOT have a dedicated ghost field for the stack size passed to `is_user_region` — it uses `thread_args.user_stack_size` from `ThreadCreateArgsView` directly. This is semantically correct (it's the same value), but the approach differs from `user_stack_base_addr` which has its own dedicated ghost field.
  **Suggested Fix:** No action required — this is consistent and correct. The stack size is already tracked in `ThreadCreateArgsView.user_stack_size`.

## New Issues Introduced

None. The changes are conservative additions of ghost parameters and documentation. No existing postconditions were weakened, no new `assume` or `external_body` constructs were introduced, and the verification still passes 18/18.

## Positive Observations

- **Ghost address traceability is now comprehensive:** All five validation call sites (`is_user_region` × 2, `is_user_addr` × 3) now thread ghost addresses through external bodies, and all ghost addresses are stored in `CreateThreadInputView`. This enables meaningful cross-module composition.

- **`IRRELEVANT_PM_OUTCOME()` is a clean abstraction:** The named spec function with documentation (spec lines 276–285) makes the early-return ghost code self-documenting. The `lemma_short_circuit_on_validation_failure` proof backs its correctness.

- **`ErrorCode::InvalidArgument as i32` is cleaner and safer:** Using the enum variant rather than a raw literal ties the exec code to the actual `ErrorCode` definition, so any future discriminant change would be caught at compile time.

- **Documentation quality remains exemplary:** The updated "Abstraction Correctness" section (exec lines 115–147) was revised to reflect the ghost address parameters, with per-step call site descriptions (lines 121–126). The API mapping table (line 153) was updated. Trust boundary descriptions are current.

- **Verification passes cleanly:** 18 verification conditions, 0 errors.

- **All previous positive observations still hold:** Comprehensive pipeline modeling, rich proof coverage (14 lemmas), clean spec/proof/exec separation, bidirectional success condition, PID ghost identity tracking.

## Summary

The prover genuinely addressed 6 of 7 previous issues. The primary HIGH issue (ghost address parameters) was substantively improved — ghost addresses are now threaded through all external body calls and exposed in postconditions, enabling cross-module linking. However, intra-module enforcement of the boolean-to-address relationship remains trusted by design. The remaining MEDIUM issue (`spec_is_valid_error_code` permissiveness) is an intentional design choice with documented rationale and a bridge lemma.

The LOW issues (hardcoded `22i32`, dummy PM outcome, missing ghost fields) were all cleanly fixed. Documentation was added for the architecture dependency and omitted fields. No new issues were introduced by the changes.

This is a well-executed verification with honest documentation of its trust boundaries. The grade improves from A- to A, reflecting the meaningful reduction in the abstraction gap and improved code quality.
