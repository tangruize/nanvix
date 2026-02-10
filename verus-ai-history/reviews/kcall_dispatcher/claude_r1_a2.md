# Review: kcall_dispatcher (claude-opus-4.6) — Round 2

## Grade: A-

## Previous Issues Assessment

### High #1: `do_kcall()` trivial postconditions — MOSTLY FIXED

**Previous:** `do_kcall` returned `i64` with tautological postcondition `result >= i64::MIN`.

**Updated (exec:465-476):** Now returns `DispatchResult` and has three postconditions:
1. `result.wf()` — well-formedness (error values in i32 range).
2. `LocalImmediate ==> result.is_success` — GetPid/GetTid always succeed.
3. `spec_do_kcall_result_category(args@) =~= spec_classify_kcall(args.number)` — result category matches classification.

**Verdict:** Postconditions #1 and #2 are meaningful improvements. However, postcondition #3 is **tautological**: `spec_do_kcall_result_category` (spec:497-499) is defined as `spec_classify_kcall(args.number as u32)`, so the postcondition reduces to `spec_classify_kcall(n) == spec_classify_kcall(n)`. This provides zero verification value — it's a self-referential identity proved by definition. See new Medium issue below.

### High #2: InterruptedKilled divergent path — FIXED

**Previous:** `handle_sleep_error` returned a fabricated `-1` for the Killed path.

**Updated (exec:387-408, exec:424-432):** `handle_sleep_error` now has precondition `spec_sleep_error_returns(sleep_error.kind)` excluding InterruptedKilled. The divergent path is separated into `handle_sleep_error_killed()` with `#[verifier::external_body]` and `ensures false`. New spec function `spec_sleep_error_returns` (spec:329-331) correctly identifies non-divergent kinds. New proof `lemma_non_divergent_sleep_errors` (proof:453-458) verifies the classification.

**Verdict:** Well addressed. The `ensures false` external_body is the standard Verus pattern for modeling divergence and the `panic!` in the body makes the trust assumption defensible.

### Medium #1: Contradictory doc-comment — FIXED

**Previous:** Comment said "modeled as a success with value -1" while code had `is_success: false`.

**Updated (spec:297-303):** Comment now accurately states "Only models the two non-divergent sleep error kinds" and documents the precondition-based exclusion.

**Verdict:** Fully fixed.

### Medium #2: Direct struct construction bypassing constructor — FIXED

**Previous:** Generic path used `DispatchResult { is_success: false, value: sleep_error.error_code }`.

**Updated (exec:397-406):** All three paths now use `DispatchResult::error()`:
- Generic: `DispatchResult::error(sleep_error.error_code as i32)` (line 398)
- TimedOut: `DispatchResult::error(110i32)` (line 401)
- Killed (unreachable): `DispatchResult::error(-1i32)` (line 405)

**Verdict:** Fully fixed. The i32 type constraint is now enforced at the type level.

### Medium #3: Unused DispatchArgs — FIXED

**Previous:** `DispatchArgs` was unused dead code.

**Updated (exec:465):** `do_kcall` now takes `DispatchArgs` as its parameter, and `spec_do_kcall_result_category` (spec:497) uses `DispatchArgsView`.

**Verdict:** Fixed. The struct is now used, though see new Medium issue about the signature mismatch.

### Medium #4: KcallResult→i64 encoding not modeled — PARTIALLY FIXED

**Previous:** No encoding model existed.

**Updated (spec:470-483, proof:408-441):** New spec functions `spec_encode_result` and `spec_could_be_error` added. New proof lemmas: `lemma_encode_preserves_value`, `lemma_error_encoding_fits_i32`, `lemma_encode_injective_on_value`.

**Verdict:** The encoding is now modeled, but `spec_encode_result(r)` is defined as `r.value` (spec:471), making all three lemmas trivially true by definition:
- `lemma_encode_preserves_value`: `r.value == r.value` ✓ (tautology)
- `lemma_error_encoding_fits_i32`: follows directly from `spec_result_wf` definition
- `lemma_encode_injective_on_value`: `r1.value == r2.value ==> r1.value == r2.value` ✓ (tautology)

The encoding model is structurally correct (it matches how `Into<i64>` works for `KcallResult`) but the lemmas provide no real assurance beyond restating definitions.

### Low #1: Empty proof bodies — ACCEPTED (unchanged)

Still present. New lemmas also have empty bodies. This is inherent to the definitional nature of the properties being verified.

### Low #2: `handle_sleep_error` takes `&SleepError` — FIXED

**Updated (exec:387):** Now takes `SleepError` by value, matching the original.

### Low #3: Spec constants as functions — ACCEPTED (unchanged)

Correctly kept as-is per Verus conventions.

## New Issues Found

### Medium

- **Location:** `do_kcall()` postcondition #3 (exec: dispatcher.rs:471)
  **Description:** The postcondition `spec_do_kcall_result_category(args@) =~= spec_classify_kcall(args.number)` is tautologically true. `spec_do_kcall_result_category` (spec:497-499) is defined as `spec_classify_kcall(args.number as u32)`. Since `args.number` is already `u32`, this postcondition reduces to `spec_classify_kcall(x) == spec_classify_kcall(x)` — a self-identity that Verus trivially proves. Similarly, the proof lemma `lemma_result_category_consistent` (proof:498-510) proves this same tautology. Neither provides verification value.
  **Suggested Fix:** Replace with a postcondition that actually constrains the result based on the category, e.g., `spec_classify_kcall(args.number) =~= DispatchCategory::LocalTerminal ==> !result.is_success` (terminal calls always fail from the caller's perspective since the process exits).

- **Location:** `do_kcall()` signature (exec: dispatcher.rs:465)
  **Description:** The original function is `pub extern "C" fn do_kcall(number: u32, arg0: u32, arg1: u32, arg2: u32, arg3: u32) -> i64`. The verified version is `pub fn do_kcall(args: DispatchArgs) -> (result: DispatchResult)`. Both the parameter passing convention (struct vs individual args) and return type (DispatchResult vs i64) differ. As an `external_body` this is a modeling choice, but it introduces a representational gap: the verified contract operates on `DispatchResult` (which has an `is_success` flag), while the actual ABI returns a raw `i64` with no such flag. The postcondition `result.is_success` for LocalImmediate calls is meaningful in the model but doesn't directly constrain the actual i64 return value.
  **Suggested Fix:** No code change strictly needed (this is inherent to the modeling approach), but document the representation gap in the trust boundary notes. Alternatively, keep the raw `(u32, u32, u32, u32, u32) -> i64` signature and express postconditions in terms of the i64 encoding.

- **Location:** `DispatchArgs::wf()` (spec: dispatcher.spec.rs:420-422)
  **Description:** The well-formedness predicate for `DispatchArgs` is still trivially `true`. Now that `DispatchArgs` is used as the parameter to `do_kcall`, this means `do_kcall` has no constraints on its input arguments. A non-trivial `wf()` could express, e.g., that the number falls within the defined kcall range, which would strengthen downstream reasoning.
  **Suggested Fix:** Consider `self.number <= 31 || self.number == u32::MAX` or keep `true` but document that all u32 values are intentionally valid (since the dispatcher's wildcard arm handles arbitrary numbers).

### Low

- **Location:** `handle_sleep_error` Killed branch (exec: dispatcher.rs:403-406)
  **Description:** The `InterruptedKilled` match arm in `handle_sleep_error` returns `DispatchResult::error(-1i32)` and is annotated as "Unreachable: excluded by precondition." This dead code is necessary for Rust's exhaustive matching but the specific return value (`-1`) is arbitrary. If someone copies this code or relaxes the precondition, the `-1` error code would be used without corresponding to any `ErrorCode`. This is safe in the current model but could be confusing.
  **Suggested Fix:** No change required — the precondition correctly excludes this path. The comment clearly marks it as unreachable.

- **Location:** Encoding lemmas (proof: dispatcher.proof.rs:408-441)
  **Description:** All three encoding lemmas (`lemma_encode_preserves_value`, `lemma_error_encoding_fits_i32`, `lemma_encode_injective_on_value`) are trivially true from definitions. While they document intent, they add verification count (3 of 49 conditions) without proving non-obvious properties.
  **Suggested Fix:** Acceptable as documentation. Could note in comments that these are definitional rather than deep properties.

## Positive Observations

- **Meaningful postconditions on do_kcall.** The `result.wf()` postcondition ensures error values fit in i32 range, and the LocalImmediate success postcondition captures a real property of GetPid/GetTid. These enable downstream verification that was previously impossible.
- **Clean divergence modeling.** The separation of `handle_sleep_error` (non-divergent, fully verified) from `handle_sleep_error_killed` (external_body divergent trust boundary) is the correct Verus idiom. The `spec_sleep_error_returns` predicate cleanly defines the partition, and `lemma_non_divergent_sleep_errors` proves its correctness.
- **Constructor usage enforces type safety.** Switching to `DispatchResult::error()` throughout `handle_sleep_error` means the i32 range constraint is enforced at the type level, not just via preconditions.
- **Verification increased from 42 to 49 conditions** with 0 errors, confirming the new lemmas and postconditions are sound.
- **API mapping table updated.** The module doc now accurately reflects the killed path separation (exec lines 54-56).
- **Good trust boundary documentation.** T4 in the module doc (exec lines 70-73) now accurately describes the divergent path modeling.
- **All previous High issues substantively addressed.** The two most significant concerns from Round 1 received genuine fixes, not just cosmetic changes.

## Summary

The prover has made substantive improvements addressing 7 of 9 previous issues (2 fully fixed High, 3 fully fixed Medium, 2 fixed Low). The verification is now materially stronger: `do_kcall` has real postconditions, the divergent Killed path is properly modeled, error handling uses typed constructors, and the encoding is modeled.

The remaining concerns are primarily about the depth of properties verified:
1. The `spec_do_kcall_result_category` postcondition on `do_kcall` is tautological and should be replaced with a property that actually constrains the result.
2. The encoding lemmas are definitionally true and provide documentation value rather than verification value.
3. The `do_kcall` signature diverges from the original ABI, creating a representation gap in the trust boundary.

These are Medium-to-Low issues that don't undermine the soundness of what IS verified. The core verification — dispatch classification, partition properties, sleep error handling, and result well-formedness — is solid and correctly specified.

**Remaining recommendations (prioritized):**
1. Replace the tautological `spec_do_kcall_result_category` postcondition with one that constrains behavior for additional categories (e.g., LocalTerminal always returns error).
2. Document the representation gap between the verified `DispatchArgs/DispatchResult` types and the actual C ABI `(u32×5) → i64`.
3. Consider whether `DispatchArgs::wf()` should be trivially true or express a meaningful constraint.
