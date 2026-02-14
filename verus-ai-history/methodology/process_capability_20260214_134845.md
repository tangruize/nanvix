# Spec Methodology Report: capability

**Directory:** /home/ubuntu/nanvix/verus/split/kernel/pm/process
**Total issues:** 5

## HIGH (3)

1. **Step 1** [view()] at `fn view()`
   - view() should be `pub closed spec fn`, not `open`
   - Guideline: view() is public but closed so users can't see internals.

2. **Step 2** [Set<T>] at `impl Set<T>`
   - No inv()/wf() function found
   - Guideline: Step 2: Write inv() as pub closed spec fn for implementation invariants.

3. **Step 2** [Capabilities] at `impl Capabilities`
   - No inv()/wf() function found
   - Guideline: Step 2: Write inv() as pub closed spec fn for implementation invariants.

## MEDIUM (2)

1. **Step 1** [CapabilitiesView] at `struct CapabilitiesView`
   - View type uses concrete type `u8` instead of `int`
   - Guideline: View types should use int instead of i32/u64 etc.

2. **Step 2** [Capabilities] at `fn wf()`
   - wf() is `open spec fn` but guideline says `pub closed spec fn`
   - Guideline: inv() should be pub closed so users can't see implementation internals.
