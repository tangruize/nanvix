# Review: thread_state Exec Consistency (claude-opus-4.6)

## Grade: A

## Verification Status

- **Result:** 47 verified, 0 errors — PASSED.
- **Command:** `./verus-ai/scripts/verify.sh thread_state`

## Issues Found

### Critical

- None.

### Minor

1. **`take_mutex_guard` return type eliminated.** The original returns
   `Option<MutexGuard>` (allowing callers to handle the not-found case
   defensively). The Verus version returns `()` and converts the not-found
   case into a precondition (`old(self)@.has_mutex(address as int)`). This
   is strictly stronger than the original: callers must prove at verification
   time that they hold the mutex, eliminating the `None` path by
   construction. The divergence is documented as Trust Assumption T2
   (lines 62–67 of state.rs) and is sound for verified callers. Unverified
   callers at the trust boundary are responsible for ensuring the invariant
   holds at runtime.

2. **`store_mutex_guard` precondition not present in original.** The original
   `BTreeMap::insert` silently overwrites on duplicate keys. The Verus model
   requires `!old(self)@.has_mutex(address as int)`, converting what was a
   silent overwrite into a proof obligation (Trust Assumption T1, lines
   57–60). This is a justified strengthening: double-locking the same mutex
   causes a deadlock in the kernel, so the precondition formalizes a kernel
   invariant that always holds in correct executions.

3. **Drop enforcement not verified at call sites.** `check_drop_safe()`
   correctly models the detection mechanism from `Drop::drop()`, and
   `lemma_check_drop_safe_models_drop` proves equivalence under `wf()`.
   However, verification does not prove that every thread reaches a
   drop-safe state before destruction. This is explicitly documented as
   requiring protocol-level verification of callers (lines 74–77).

## Function-by-Function Assessment

| Original Function | Exec Status | Equivalence |
|--------------------|-------------|-------------|
| `new` | Present | ✅ Equivalent — same initialization; `context`/`fpu_state` params elided (opaque HAL). |
| `id` | Present | ✅ Identical — returns `self.id`. |
| `set_interrupt_reason` | Present | ✅ Equivalent — `self.interrupt_reason = Some(reason)`. Type `InterruptReason` → `int`. |
| `take_interrupt_reason` | Present | ✅ Equivalent — expanded `Option::take()` (Verus limitation). |
| `take_kernel_stack` | Present | ✅ Equivalent — expanded `Option::take()`. Identity preserved. |
| `take_user_stack` | Present | ✅ Equivalent — expanded `Option::take()`. Identity preserved. |
| `store_mutex_guard` | Present | ✅ Protocol model — `BTreeMap::insert` → `Vec::push` + counter. Frame condition proven. |
| `take_mutex_guard` | Present | ⚠️ Protocol model — `BTreeMap::remove` → linear search + `Vec::remove`. Return type eliminated (T2). |
| `store_thread_data_area` | Present | ✅ Identical — `self.user_tda = user_tda`. Type `VirtualAddress` → `int`. |
| `get_thread_data_area` | Present | ✅ Identical — returns `self.user_tda`. |
| `context_mut` | Omitted | ✅ Sound — returns raw pointer via Pin projection; opaque HAL boundary. |
| `fpu_state_mut` | Omitted | ✅ Sound — same rationale as `context_mut`. |
| `join_cond` | Omitted | ✅ Sound — `Condvar` with interior mutability; unmodeled sync primitive. |
| `Debug::fmt` | Omitted | ✅ Sound — display-only trait, no state semantics. |
| `Drop::drop` | Modeled by `check_drop_safe` | ✅ Sound — `lemma_check_drop_safe_models_drop` proves equivalence under `wf()`. |
| `check_drop_safe` | Added | ✅ Justified — exec helper modeling `Drop::drop()` runtime check. |

## Type Abstraction Assessment

| Original Type | Verus Model | Soundness |
|--------------|-------------|-----------|
| `KernelStack` | `Option<int>` | ✅ Abstract resource token; identity preserved via Option::take. |
| `UserStack` | `Option<int>` | ✅ Same rationale. |
| `VirtualAddress` | `int` | ✅ Abstract address; only identity matters. |
| `InterruptReason` | `int` | ✅ Abstract reason tag; only presence matters. |
| `BTreeMap<MutexAddress, MutexGuard>` | `Vec<u64>` + `locked_mutex_count` + ghost `Set<int>` | ✅ Per-key semantics faithfully modeled via `seq_to_set`. |
| `ContextInformation` | Elided | ✅ Opaque HAL type; out of verification scope. |
| `FpuState` | Elided | ✅ Same rationale. |
| `Condvar` | Elided | ✅ Opaque sync primitive with interior mutability. |
| `MutexGuard` | Elided (address-only tracking) | ✅ Guard payload opaque; only lock accounting verified. |

## Proof Quality Assessment

- **47 verified properties** covering: construction well-formedness, ID immutability
  across all operations, Option take/store semantics, mutex guard per-address
  non-interference, well-formedness preservation, drop safety detection,
  mutex store/take roundtrip, view equality, and resource tracking.
- `wf()` is `closed spec` — appropriately encapsulated with `reveal()` in
  exec and proof contexts.
- Frame conditions (field preservation across mutations) are comprehensive:
  every exec function ensures all unmodified fields are preserved.
- The `locked_mutex_set` Vec-to-Set bridge via `seq_to_set` with
  `no_duplicates()` invariant is well-structured and the supporting lemmas
  (`lemma_seq_to_set_*`) provide the necessary proof infrastructure.

## Summary

The exec consistency fix is well-executed. All 10 original functions that
involve state management protocol logic are present in the Verus exec code
with semantically equivalent implementations. The 5 omitted functions
(`context_mut`, `fpu_state_mut`, `join_cond`, `fmt`, `drop`) are correctly
identified as out of verification scope — they involve opaque HAL types,
raw pointer safety, Pin projections, or sync primitives that cannot be
meaningfully modeled in pure spec. The `Drop::drop` invariant is faithfully
captured by `check_drop_safe()` with proven equivalence under `wf()`.

The type abstractions (stacks → `Option<int>`, mutexes → `Vec<u64>` +
ghost `Set<int>`) are sound choices that enable verification of the state
management protocol while correctly scoping out opaque kernel subsystem
types. The two trust assumptions (T1: no double-lock, T2: release-what-you-hold)
are well-documented and formally encode kernel invariants that hold in
correct executions.

The primary area for future improvement is verifying drop safety enforcement
at call sites (i.e., proving that all thread termination paths reach a
drop-safe state), which would close the gap between detection and enforcement.
This is correctly documented as out of current scope.
