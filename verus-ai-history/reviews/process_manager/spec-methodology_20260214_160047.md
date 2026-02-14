# Review: process_manager Spec Methodology (claude-opus-4.6)

## Grade: B+

## Issues Found

### Critical
- None.

### High

- **H1: `view()` is `open spec fn` instead of `pub closed spec fn`.**
  Both `ProcessManagerInner` (process_manager.spec.rs:537) and `ProcessManagerUnsafeState`
  (process_manager_unsafe.spec.rs:200) declare `view()` as `open spec fn` via the
  `impl View for ...` trait. The guidelines (specifying-and-proving-types.md, Step 1)
  require `pub closed spec fn view(&self) -> MyTypeView` so that users cannot reason
  about implementation internals. The `open` visibility exposes the mapping from concrete
  fields to the View type, allowing callers to bypass the abstraction boundary.
  **Note:** Verus's `View` trait may impose `open` as a constraint for trait impl items.
  If so, this is a framework limitation, not a code defect — but should be documented.

- **H2: `wf()` is `pub open spec fn` instead of `pub closed spec fn`.**
  Both `ProcessManagerInner::wf()` (process_manager.spec.rs:463) and
  `ProcessManagerUnsafeState::wf()` (process_manager_unsafe.spec.rs:161) are declared
  `pub open`. The guidelines (Step 2) require `pub closed spec fn inv(&self) -> bool`
  so that users cannot see implementation invariant details. Making `wf()` open means
  callers can unfold the invariant and depend on internal structural properties (e.g.,
  ghost set disjointness, PID bounds), coupling external code to implementation details.

- **H3: Public method specs expose implementation fields (`self.field`) instead of
  using view-based abstraction (`self@.field`).**
  All public methods on `ProcessManagerInner` (e.g., `create_process`, `schedule`,
  `sleep_running`, `exit_running`, etc.) use direct field access in their `ensures`
  clauses: `self.running_pid`, `self.ghost_ready@`, `self.ready_count`,
  `self.next_pid`, `self.number_buffered_messages`, `self.interrupt_capable`, etc.
  The guidelines (Step 3) require public method specs to use `self@.field` (i.e.,
  `self.view().field`) instead of `self.field` to maintain abstraction. For example,
  `create_process` ensures `self.next_pid as int == old(self).next_pid as int + 1`
  but should say `self@.next_pid == old(self)@.next_pid + 1`. Similarly for
  `ProcessManagerUnsafeState` public methods which reference `self.inner`,
  `self.current_pid`, `self.remaining_quantum`, etc. directly.

### Medium

- **M1: Many spec helper functions on `ProcessManagerInner` are `pub open` when they
  should be private or `pub closed`.**
  The guidelines (Step 3) state: "Don't write any further `pub` specification functions
  in `impl MyType` beyond `inv` and `view`." However, `ProcessManagerInner` exposes 19
  additional `pub open spec fn` helpers (e.g., `spec_running_pid`, `spec_sets_finite`,
  `spec_counts_match`, `spec_queues_disjoint`, `spec_running_exclusive`,
  `spec_kernel_safe`, `spec_pid_bounds`, `spec_counts_bounded`, `spec_no_dups`,
  `spec_process_exists`, `spec_has_ready`, `spec_has_zombies`, `spec_pid_is_fresh`,
  `spec_can_create_process`, `spec_ready_with_running`, `spec_full_schedule_pool`).
  These expose internal invariant structure. Per the guidelines, spec helpers that
  abbreviate common expressions for *public* method specs should be `pub open spec fn`
  on the *View type* (not on the impl type), while internal helpers should be private.

- **M2: View type uses `pub` fields, which is acceptable in Verus spec types.**
  The `ProcessManagerInnerView` and `ProcessManagerUnsafeStateView` structs have all
  fields `pub`. This is standard for Verus view/spec types and is not an issue, but
  worth noting that this differs from the Nanvix coding standard for exec types
  (where fields should be private with getters/setters).

### Low

- **L1: Naming uses `wf()` instead of `inv()` from the guidelines.**
  The guidelines reference `inv()` as the invariant function name. The code uses `wf()`
  (well-formedness). This is a cosmetic divergence — the semantics are correct and `wf`
  is a reasonable alternative name. No functional impact.

- **L2: View type spec transition functions are `pub open` on `ProcessManagerInnerView`,
  which is correct per guidelines.**
  The guidelines (Step 3, last paragraph) say: "create a `pub open spec fn` in
  `MyTypeView` for each such common expression." The 11 `spec_*` transition functions
  on `ProcessManagerInnerView` (e.g., `spec_create_process`, `spec_schedule`, etc.)
  follow this pattern correctly.

## Verification Status

- **ProcessManagerInner**: 112 verified, 0 errors ✅
- **ProcessManagerUnsafeState**: 40 verified, 0 errors ✅
- **No `assume`, `admit`, or unjustified `external_body`** found in any file. ✅

## View Type Assessment

The View types use appropriate abstract types:
- `int` for PIDs (instead of `i32`) ✅
- `Set<int>` for queue membership (instead of `Vec<u64>` / `PidSet`) ✅
- `nat` for counts and message counts (instead of `usize`) ✅
- `bool` for flags (same in both) ✅

The `ProcessManagerInnerView` correctly omits internal implementation details like
`ready_count`, `suspended_count`, etc. (these are derivable from set cardinality).
The `ProcessManagerUnsafeStateView` uses `ProcessManagerInnerView` for composition. ✅

## Summary

The process_manager verification is comprehensive (152 total verified functions, 0 errors)
with no assumes or admits. The View types correctly use abstract types and the transition
spec functions on the View type follow the guidelines well. However, the code has
significant methodology deviations from the specifying-and-proving-types guidelines:
`view()` and `wf()` are `open` instead of `closed`, public method specs reference
implementation fields directly (`self.field`) instead of through the view abstraction
(`self@.field`), and many internal spec helpers are unnecessarily `pub open` on the
impl type rather than being private or placed on the View type. These issues mean that
external code can depend on implementation internals, undermining the abstraction
boundary that the View type is designed to provide. The verification itself is sound
and thorough, but the spec methodology would benefit from tightening the visibility
boundaries to match the guidelines.
