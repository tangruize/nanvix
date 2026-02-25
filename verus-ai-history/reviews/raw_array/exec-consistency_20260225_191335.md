# Review: raw_array Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Status
- **Result:** 21 verified, 0 errors
- **Command:** `verus --crate-type lib lib.rs --verify-module libs::raw_array`
- **Note:** The `./verus-ai/scripts/verify.sh raw_array` command fails due to a module resolution bug — the script does not correctly resolve `raw_array` to `libs::raw_array` because the file is `libs/raw_array/lib.rs` (not `libs/raw_array.rs`). Running Verus directly with the correct module path succeeds.

## Function-by-Function Comparison

| Original Function | Verus Status | Faithful? | Notes |
|---|---|---|---|
| `RawArrayStorage` enum | Present (lib.rs:30-33) | ✅ Exact | Same variants and fields |
| `RawArrayStorage::new_managed` | Present (lib.rs:36-62) | ✅ Exact | Body identical line-for-line |
| `RawArrayStorage::new_unmanaged` | Present (lib.rs:64-85) | ✅ Exact | Body identical line-for-line |
| `RawArrayStorage::get_mut` | Present (lib.rs:87-96) | ✅ Exact | Body identical |
| `RawArrayStorage::get` | Present (lib.rs:98-107) | ✅ Exact | Body identical |
| `RawArray<T>` struct | Present (lib.rs:145-148) | ✅ Exact | Same field; doc comment restored |
| `RawArray::new` | Present (lib.rs:157-170) | ✅ Exact | Same body; added `requires`/`ensures` |
| `RawArray::from_raw_parts` | Present (lib.rs:174-187) | ✅ Exact | Same body; added `requires`/`ensures` |
| `Deref for RawArray<T>` | Present (lib.rs:242-252) | ✅ Exact | Same body; added `ensures` |
| `DerefMut for RawArray<T>` | Present (lib.rs:258-261) | ✅ Exact | Outside `verus!{}` — correct |
| `Drop for RawArray<T>` | Moved to `Drop for RawArrayStorage<T>` (lib.rs:114-129) | ✅ Equivalent | See analysis below |

## Detailed Analysis

### 1. MISMATCH Functions — Properly Restored?
**Yes.** The consistency report identified one mismatch: the missing doc comment on the `storage` field of `RawArray`. This has been restored at lib.rs:147 (`/// The backing storage of the raw array.`), matching the original at src/libs/raw-array/src/lib.rs:206.

### 2. MISSING Functions — Added with Proper Verification?
**N/A.** The report states 0 missing functions were added. The only originally-missing function (`Drop for RawArray<T>`) was handled via equivalence documentation rather than direct addition, which is the correct approach (see below).

### 3. Equivalence Justifications — Sound?

**Drop relocation (RawArray → RawArrayStorage):** Sound. Verus does not support trait impls inside `verus!{}` blocks. Moving `Drop` from `RawArray` to `RawArrayStorage` is semantically equivalent because:
- When `RawArray` is dropped, Rust drops its fields, triggering `RawArrayStorage::drop`.
- The deallocation logic is identical: same `Layout::array::<T>(*len)` computation, same `dealloc` call for `Managed`, same no-op for `Unmanaged`.
- The only syntactic difference is `match &self.storage` (original) vs `match self` (Verus), which is a necessary structural consequence of the move.
- Adding both would cause double-free. ✅

**Verification helpers (`set`, `len`, `get`):** Sound. The original source uses `Deref`/`DerefMut` trait dispatch for element access (e.g., `self[index]`). Verus cannot verify through trait dispatch, so these `external_body` helpers provide explicit verified accessors with `requires`/`ensures` contracts. Their implementations delegate to `self.storage.get()` / `self.storage.get_mut()`, which is exactly what the `Deref`/`DerefMut` impls do. ✅

**`ExRawArrayStorage`:** Sound. This is a standard Verus pattern (`#[verifier::external_type_specification]`) required to create a spec-level wrapper for `RawArrayStorage` which is defined outside `verus!{}`. It introduces no runtime behavior. ✅

### 4. Exec Code Faithfulness
The exec code faithfully represents the original source. Every function body in the Verus version is character-for-character identical to the original, with the sole structural difference being the `Drop` relocation (well-justified). The `#[verifier::external_body]` annotations on constructors and accessors are appropriate — they tell Verus to trust the exec body and verify only through the contracts.

### 5. Verification Status
Passes: 21 verified, 0 errors. The proof and test files contain substantive lemmas (update preservation, frame conditions, equality properties) that exercise the spec-level model.

## Issues Found

### Critical
- None.

### Minor
- **Verify script resolution bug:** `./verus-ai/scripts/verify.sh raw_array` fails because the script cannot resolve `raw_array` to `libs::raw_array` when the file is `libs/raw_array/lib.rs`. This is a tooling issue, not a code issue. Direct invocation of Verus succeeds.
- **Missing `#[derive(Debug)]`:** The original has `#[derive(Debug)]` on both `RawArrayStorage` and `RawArray`. The Verus version omits these. Not functionally significant but could affect debugging. Verus may not support derive macros inside `verus!{}` blocks, so this is likely intentional.
- **Stripped doc comments on internal functions:** `RawArrayStorage` methods (`new_managed`, `new_unmanaged`, `get_mut`, `get`) lack the `///` doc comments present in the original. These are private implementation details so the impact is low, but completeness would be improved by restoring them.

## Summary

The exec consistency fixes are well-executed. All original functions are faithfully reproduced with identical bodies. The `Drop` relocation from `RawArray` to `RawArrayStorage` is the only structural deviation and is thoroughly justified — it avoids a Verus limitation while preserving identical deallocation semantics. The verification helpers (`set`, `len`, `get`) are necessary additions that bridge the gap between Rust's trait dispatch (which Verus cannot verify) and Verus's contract-based verification. Verification passes cleanly with 21 verified, 0 errors. Grade is A rather than A+ due to minor doc comment omissions and the verify script resolution issue.
