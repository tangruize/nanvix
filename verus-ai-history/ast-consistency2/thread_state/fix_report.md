# Exec Consistency Fix: thread_state

## Summary
- Mismatches fixed: 0 (all 10 documented as semantically equivalent)
- Missing functions added: 0 (all 5 documented as out of verification scope)
- Documented equivalences: 16

## Analysis

### Verification Model

The Verus `ThreadState` is an **abstract verification model** of the original
`src/kernel/src/pm/thread/state.rs::ThreadState`. Complex kernel types are
intentionally abstracted (see module-level documentation in `state.rs`):

- `KernelStack`, `UserStack` → `Option<int>` (abstract resource tokens)
- `InterruptReason` → `Option<int>` (abstract reason tag)
- `BTreeMap<MutexAddress, MutexGuard>` → `Vec<u64>` + `locked_mutex_count: usize`
  with ghost `Set<int>` (protocol-only model)
- `Pin<Box<ContextInformation>>`, `Pin<Box<FpuState>>`, `Condvar` → elided
  (opaque HAL/sync boundary types)

All exec logic differences stem from these type abstractions. No executable
logic was changed in any function.

## Changes

| Function | Action | Justification |
|----------|--------|---------------|
| `new` [new.diff](new.diff) [new_source.rs](new_source.rs) [new_verus.rs](new_verus.rs) | Documented equivalence | Same initialization pattern; parameter types differ due to model abstraction (`ContextInformation`/`FpuState` elided as opaque HAL types, `KernelStack`/`UserStack` → `Option<int>`). All fields initialized to same logical values: stacks/tda from params, interrupt_reason=None, locked_mutexes=empty. |
| `id` [id.diff](id.diff) [id_source.rs](id_source.rs) [id_verus.rs](id_verus.rs) | Documented equivalence | Identical logic: `self.id`. Only difference is Verus `ensures` annotations. |
| `set_interrupt_reason` [set_interrupt_reason.diff](set_interrupt_reason.diff) [set_interrupt_reason_source.rs](set_interrupt_reason_source.rs) [set_interrupt_reason_verus.rs](set_interrupt_reason_verus.rs) | Documented equivalence | Identical logic: `self.interrupt_reason = Some(reason)`. Parameter type `InterruptReason` → `int` due to model abstraction. |
| `take_interrupt_reason` [take_interrupt_reason.diff](take_interrupt_reason.diff) [take_interrupt_reason_source.rs](take_interrupt_reason_source.rs) [take_interrupt_reason_verus.rs](take_interrupt_reason_verus.rs) | Documented equivalence | Semantically identical to `Option::take()`: saves value, sets field to None, returns saved value. Expanded form required because Verus does not support `.take()` method on `Option`. |
| `take_kernel_stack` [take_kernel_stack.diff](take_kernel_stack.diff) [take_kernel_stack_source.rs](take_kernel_stack_source.rs) [take_kernel_stack_verus.rs](take_kernel_stack_verus.rs) | Documented equivalence | Semantically identical to `Option::take()`: saves value, sets field to None, returns saved value. Same Verus limitation as above. |
| `take_user_stack` [take_user_stack.diff](take_user_stack.diff) [take_user_stack_source.rs](take_user_stack_source.rs) [take_user_stack_verus.rs](take_user_stack_verus.rs) | Documented equivalence | Semantically identical to `Option::take()`: saves value, sets field to None, returns saved value. Same Verus limitation as above. |
| `store_mutex_guard` [store_mutex_guard.diff](store_mutex_guard.diff) [store_mutex_guard_source.rs](store_mutex_guard_source.rs) [store_mutex_guard_verus.rs](store_mutex_guard_verus.rs) | Documented equivalence | Models `BTreeMap::insert(address, guard)` as `Vec::push(address)` + counter increment. Protocol-only model: tracks which addresses are held (not guard values). Precondition `!has_mutex(address)` formalizes no-double-lock kernel invariant (Trust Assumption T1). |
| `take_mutex_guard` [take_mutex_guard.diff](take_mutex_guard.diff) [take_mutex_guard_source.rs](take_mutex_guard_source.rs) [take_mutex_guard_verus.rs](take_mutex_guard_verus.rs) | Documented equivalence | Models `BTreeMap::remove(&address)` as linear search + `Vec::remove`. Precondition `has_mutex(address)` converts runtime `Option` return to proof obligation (Trust Assumption T2), eliminating None path by construction. |
| `store_thread_data_area` [store_thread_data_area.diff](store_thread_data_area.diff) [store_thread_data_area_source.rs](store_thread_data_area_source.rs) [store_thread_data_area_verus.rs](store_thread_data_area_verus.rs) | Documented equivalence | Identical logic: `self.user_tda = user_tda`. Type `Option<VirtualAddress>` → `Option<int>`. |
| `get_thread_data_area` [get_thread_data_area.diff](get_thread_data_area.diff) [get_thread_data_area_source.rs](get_thread_data_area_source.rs) [get_thread_data_area_verus.rs](get_thread_data_area_verus.rs) | Documented equivalence | Identical logic: `self.user_tda`. Return type `Option<VirtualAddress>` → `Option<int>`. |
| `context_mut` [context_mut_source.rs](context_mut_source.rs) | Omitted (out of scope) | Returns `*mut ContextInformation` via `Pin<Box<_>>` projection. Involves raw pointer safety and Pin projection — out of verification scope (documented in module header lines 79-80). Type does not exist in verification model. |
| `fpu_state_mut` [fpu_state_mut_source.rs](fpu_state_mut_source.rs) | Omitted (out of scope) | Returns `*mut FpuState` via `Pin<Box<_>>` projection. Same raw pointer/Pin scope exclusion as `context_mut`. |
| `join_cond` [join_cond_source.rs](join_cond_source.rs) | Omitted (out of scope) | Returns `self.join_cond.clone()`. `Condvar` is an opaque sync primitive with interior mutability — cannot be meaningfully modeled in pure spec (documented in module header line 81). |
| `fmt` [fmt_source.rs](fmt_source.rs) | Omitted (out of scope) | `Debug` trait impl for display only. Not part of state management protocol. No verifiable properties. |
| `drop` [drop_source.rs](drop_source.rs) | Omitted (modeled by `check_drop_safe`) | `Drop::drop()` checks `!self.locked_mutexes.is_empty()` and logs error. Verification models this via `check_drop_safe()` exec function + `spec_drop_safe()` spec + `lemma_check_drop_safe_models_drop` proof (equivalence proven under `wf()`). See module header lines 22-27. |
| `check_drop_safe` [check_drop_safe_verus.rs](check_drop_safe_verus.rs) | Retained (justified helper) | Exec-level function modeling the runtime check from `Drop::drop()`. Returns `self.locked_mutex_count == 0`. Proven equivalent to `spec_drop_safe()` under `wf()` by `lemma_check_drop_safe_models_drop`. Documented in module header lines 69-77. |

## Struct: `ThreadState`

The struct MISMATCH is inherent to the verification model design:

| Original Field | Verus Field | Model |
|---------------|-------------|-------|
| `id: ThreadIdentifier` | `id: ThreadIdentifier` | Same (verified dependency) |
| `kernel_stack: Option<KernelStack>` | `kernel_stack: Option<int>` | Abstract resource token |
| `user_stack: Option<UserStack>` | `user_stack: Option<int>` | Abstract resource token |
| `user_tda: Option<VirtualAddress>` | `user_tda: Option<int>` | Abstract address |
| `interrupt_reason: Option<InterruptReason>` | `interrupt_reason: Option<int>` | Abstract reason tag |
| `locked_mutexes: BTreeMap<MutexAddress, MutexGuard>` | `locked_mutex_count: usize` + `locked_mutex_set: Vec<u64>` | Protocol model with ghost set |
| `join_cond: Condvar` | *(elided)* | Opaque sync primitive |
| `context: Pin<Box<ContextInformation>>` | *(elided)* | Opaque HAL type |
| `fpu_state: Pin<Box<FpuState>>` | *(elided)* | Opaque HAL type |

## Verification: PASS

```
verification results:: 47 verified, 0 errors
```
